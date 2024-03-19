// Validator for Enums, so named because "enum" is a reserved keyword in Rust.
use core::fmt::Debug;

use pyo3::exceptions::PyTypeError;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyType};

use crate::build_tools::py_schema_err;
use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::Input;
use crate::tools::{safe_repr, SchemaDict};

use super::is_instance::class_repr;
use super::literal::{expected_repr_name, LiteralLookup};
use super::{BuildValidator, CombinedValidator, DefinitionsBuilder, ValidationState, Validator};

#[derive(Debug, Clone)]
pub struct BuildEnumValidator;

impl BuildValidator for BuildEnumValidator {
    const EXPECTED_TYPE: &'static str = "enum";

    fn build(
        schema: &Bound<'_, PyDict>,
        _config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let members: Bound<PyList> = schema.get_as_req(intern!(schema.py(), "members"))?;
        if members.is_empty() {
            return py_schema_err!("`members` should have length > 0");
        }

        let py = schema.py();
        let value_str = intern!(py, "value");
        let expected: Vec<(Bound<'_, PyAny>, PyObject)> = members
            .iter()
            .map(|v| Ok((v.getattr(value_str)?, v.into())))
            .collect::<PyResult<_>>()?;

        let repr_args: Vec<String> = expected
            .iter()
            .map(|(k, _)| k.repr()?.extract())
            .collect::<PyResult<_>>()?;

        let class: Bound<PyType> = schema.get_as_req(intern!(py, "cls"))?;
        let class_repr = class_repr(schema, &class)?;
        let gv = GeneralEnumValidator {
            class: class.clone().into(),
            lookup: LiteralLookup::new(py, expected.into_iter())?,
            missing_func: schema.get_as(intern!(py, "missing_func"))?,
            expected_repr: expected_repr_name(repr_args, "").0,
            class_repr: class_repr.clone(),
        };
        let sub_type: Option<String> = schema.get_as(intern!(py, "sub_type"))?;

        match sub_type.as_deref() {
            Some("int") => Ok(CombinedValidator::IntEnum(IntEnumValidator {
                gv,
                name: format!("int-enum[{class_repr}]"),
            })),
            Some("str") => Ok(CombinedValidator::StrEnum(StrEnumValidator {
                gv,
                name: format!("str-enum[{class_repr}]"),
            })),
            Some("float") => Ok(CombinedValidator::FloatEnum(FloatEnumValidator {
                gv,
                name: format!("float-enum[{class_repr}]"),
            })),
            Some(_) => py_schema_err!("`sub_type` must be one of: 'int', 'str' or None"),
            None => Ok(CombinedValidator::PlainEnum(PlainEnumValidator {
                gv,
                name: format!("{}[{class_repr}]", Self::EXPECTED_TYPE),
            })),
        }
    }
}

#[derive(Debug, Clone)]
struct GeneralEnumValidator {
    class: Py<PyType>,
    lookup: LiteralLookup<PyObject>,
    missing_func: Option<PyObject>,
    expected_repr: String,
    class_repr: String,
}

impl_py_gc_traverse!(GeneralEnumValidator {
    class,
    lookup,
    missing_func
});

impl GeneralEnumValidator {
    /// Try to match the behavior of https://github.com/python/cpython/blob/v3.12.2/Lib/enum.py#L1116
    fn validate<'py, I: Input<'py> + ?Sized>(
        &self,
        py: Python<'py>,
        input: &I,
        strict: bool,
        validate_value: impl FnOnce(&I) -> ValResult<Option<PyObject>>,
    ) -> ValResult<PyObject> {
        // exact type check as per
        let class = self.class.bind(py);
        if input.input_is_exact_instance(class) {
            Ok(input.to_object(py))
        } else if strict {
            // TODO what about instances of subclasses?
            Err(ValError::new(
                ErrorType::IsInstanceOf {
                    class: self.class_repr.clone(),
                    context: None,
                },
                input,
            ))
        } else if let Some(v) = validate_value(input)? {
            Ok(v)
        } else if let Some(ref missing_func) = self.missing_func {
            let missing_func = missing_func.bind(py);
            let enum_value = missing_func.call1((input.to_object(py),))?;
            // check enum_value is an instance of the class like
            // https://github.com/python/cpython/blob/v3.12.2/Lib/enum.py#L1148
            if enum_value.is_instance(class)? {
                Ok(enum_value.into())
            } else {
                let type_error = PyTypeError::new_err(format!(
                    "error in {}._missing_: returned {} instead of None or a valid member",
                    class.name().unwrap_or_else(|_| "<Unknown>".into()),
                    safe_repr(&enum_value)
                ));
                Err(type_error.into())
            }
        } else {
            Err(ValError::new(
                ErrorType::Enum {
                    expected: self.expected_repr.clone(),
                    context: None,
                },
                input,
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlainEnumValidator {
    gv: GeneralEnumValidator,
    name: String,
}

impl_py_gc_traverse!(PlainEnumValidator { gv });

impl Validator for PlainEnumValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        self.gv.validate(py, input, state.strict_or(false), |input| {
            Ok(self.gv.lookup.validate(py, input)?.map(|(_, v)| v.clone()))
        })
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct IntEnumValidator {
    gv: GeneralEnumValidator,
    name: String,
}

impl_py_gc_traverse!(IntEnumValidator { gv });

impl Validator for IntEnumValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        self.gv.validate(py, input, state.strict_or(false), |input| {
            Ok(self.gv.lookup.validate_int_lax(py, input)?.map(Clone::clone))
        })
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct StrEnumValidator {
    gv: GeneralEnumValidator,
    name: String,
}

impl_py_gc_traverse!(StrEnumValidator { gv });

impl Validator for StrEnumValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        self.gv.validate(py, input, state.strict_or(false), |input| {
            Ok(self.gv.lookup.validate_str_lax(input)?.map(Clone::clone))
        })
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct FloatEnumValidator {
    gv: GeneralEnumValidator,
    name: String,
}

impl_py_gc_traverse!(FloatEnumValidator { gv });

impl Validator for FloatEnumValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        self.gv.validate(py, input, state.strict_or(false), |input| {
            Ok(self.gv.lookup.validate_float_lax(py, input)?.map(Clone::clone))
        })
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}
