use std::sync::Arc;

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::intern;
use pyo3::sync::PyOnceLock;
use pyo3::types::{IntoPyDict, PyDict, PyString, PyTuple, PyType};
use pyo3::{prelude::*, PyTypeInfo};

use crate::build_tools::{is_strict, schema_or_config_same};
use crate::errors::ErrorType;
use crate::errors::ValResult;
use crate::errors::{ErrorTypeDefaults, Number};
use crate::errors::{ToErrorValue, ValError};
use crate::input::Input;
use crate::tools::SchemaDict;

use super::{BuildValidator, CombinedValidator, DefinitionsBuilder, ValidationState, Validator};

static FRACTION_TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();

pub fn get_fraction_type(py: Python<'_>) -> &Bound<'_, PyType> {
    FRACTION_TYPE
        .get_or_init(py, || {
            py.import("fractions")
                .and_then(|fraction_module| fraction_module.getattr("Fraction"))
                .unwrap()
                .extract()
                .unwrap()
        })
        .bind(py)
}

fn validate_as_fraction(
    py: Python,
    schema: &Bound<'_, PyDict>,
    key: &Bound<'_, PyString>,
) -> PyResult<Option<Py<PyAny>>> {
    match schema.get_item(key)? {
        Some(value) => match value.validate_fraction(false, py) {
            Ok(v) => Ok(Some(v.into_inner().unbind())),
            Err(_) => Err(PyValueError::new_err(format!(
                "'{key}' must be coercible to a Decimal instance",
            ))),
        },
        None => Ok(None),
    }
}

#[derive(Debug, Clone)]
pub struct FractionValidator {
    strict: bool,
    le: Option<Py<PyAny>>,
    lt: Option<Py<PyAny>>,
    ge: Option<Py<PyAny>>,
    gt: Option<Py<PyAny>>,
}

impl BuildValidator for FractionValidator {
    const EXPECTED_TYPE: &'static str = "fraction";
    fn build(
        schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<Arc<CombinedValidator>>,
    ) -> PyResult<Arc<CombinedValidator>> {
        let py = schema.py();

        let allow_inf_nan = schema_or_config_same(schema, config, intern!(py, "allow_inf_nan"))?.unwrap_or(false);

        Ok(CombinedValidator::Fraction(Self {
            strict: is_strict(schema, config)?,
            le: validate_as_fraction(py, schema, intern!(py, "le"))?,
            lt: validate_as_fraction(py, schema, intern!(py, "lt"))?,
            ge: validate_as_fraction(py, schema, intern!(py, "ge"))?,
            gt: validate_as_fraction(py, schema, intern!(py, "gt"))?,
        })
        .into())
    }
}

impl_py_gc_traverse!(FractionValidator {
    le,
    lt,
    ge,
    gt
});

fn extract_fraction_as_ints(fraction: &Bound<'_, PyAny>) -> ValResult<(i64, i64)> {
    let py = fraction.py();
    // just call fraction.numerator and fraction.denominator
    let numerator: i64 = fraction.getattr(intern!(py, "numerator"))?.extract()?;
    let denominator: i64 = fraction.getattr(intern!(py, "denominator"))?.extract()?;

    Ok((numerator, denominator))
}

impl Validator for FractionValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<Py<PyAny>> {
        let fraction = input.validate_fraction(state.strict_or(self.strict), py)?.unpack(state);

        // let mut is_nan: Option<bool> = None;
        // let mut is_nan = || -> PyResult<bool> {
        //     match is_nan {
        //         Some(is_nan) => Ok(is_nan),
        //         None => Ok(*is_nan.insert(decimal.call_method0(intern!(py, "is_nan"))?.extract()?)),
        //     }
        // };

        // if let Some(le) = &self.le {
        //     if is_nan()? || !decimal.le(le)? {
        //         return Err(ValError::new(
        //             ErrorType::LessThanEqual {
        //                 le: Number::String(le.to_string()),
        //                 context: Some([("le", le)].into_py_dict(py)?.into()),
        //             },
        //             input,
        //         ));
        //     }
        // }
        // if let Some(lt) = &self.lt {
        //     if is_nan()? || !decimal.lt(lt)? {
        //         return Err(ValError::new(
        //             ErrorType::LessThan {
        //                 lt: Number::String(lt.to_string()),
        //                 context: Some([("lt", lt)].into_py_dict(py)?.into()),
        //             },
        //             input,
        //         ));
        //     }
        // }
        // if let Some(ge) = &self.ge {
        //     if is_nan()? || !decimal.ge(ge)? {
        //         return Err(ValError::new(
        //             ErrorType::GreaterThanEqual {
        //                 ge: Number::String(ge.to_string()),
        //                 context: Some([("ge", ge)].into_py_dict(py)?.into()),
        //             },
        //             input,
        //         ));
        //     }
        // }
        // if let Some(gt) = &self.gt {
        //     if is_nan()? || !decimal.gt(gt)? {
        //         return Err(ValError::new(
        //             ErrorType::GreaterThan {
        //                 gt: Number::String(gt.to_string()),
        //                 context: Some([("gt", gt)].into_py_dict(py)?.into()),
        //             },
        //             input,
        //         ));
        //     }
        // }

        Ok(fraction.into())
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}

pub(crate) fn create_fraction<'py>(arg: &Bound<'py, PyAny>, input: impl ToErrorValue) -> ValResult<Bound<'py, PyAny>> {
    let py = arg.py();
    get_fraction_type(py).call1((arg,)).map_err(|e| {
        let fraction_exception = match py
            .import("fractions")
            .and_then(|fraction_module| fraction_module.getattr("FractionException"))
        {
            Ok(fraction_exception) => fraction_exception,
            Err(e) => return ValError::InternalErr(e),
        };
        handle_fraction_new_error(input, e, fraction_exception)
    })
}

fn handle_fraction_new_error(input: impl ToErrorValue, error: PyErr, fraction_exception: Bound<'_, PyAny>) -> ValError {
    let py = fraction_exception.py();
    if error.matches(py, fraction_exception).unwrap_or(false) {
        ValError::new(ErrorTypeDefaults::FractionParsing, input)
    } else if error.matches(py, PyTypeError::type_object(py)).unwrap_or(false) {
        ValError::new(ErrorTypeDefaults::FractionType, input)
    } else {
        ValError::InternalErr(error)
    }
}
