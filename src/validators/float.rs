use std::cmp::Ordering;
use std::sync::Arc;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::errors::{ErrorType, ErrorTypeDefaults, ValError, ValResult};
use crate::input::Input;
use crate::tools::SchemaDict;

use super::config::ConfigValidator;
use super::{BuildValidator, CombinedValidator, DefinitionsBuilder, ValidationState, Validator};

pub struct FloatBuilder;

impl BuildValidator for FloatBuilder {
    const EXPECTED_TYPE: &'static str = "float";
    fn build(
        schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let use_constrained = schema.get_item(intern!(py, "multiple_of"))?.is_some()
            || schema.get_item(intern!(py, "le"))?.is_some()
            || schema.get_item(intern!(py, "lt"))?.is_some()
            || schema.get_item(intern!(py, "ge"))?.is_some()
            || schema.get_item(intern!(py, "gt"))?.is_some();

        let mut validator = if use_constrained {
            ConstrainedFloatValidator::build(schema, config, definitions)?
        } else {
            FloatValidator::build(schema, config, definitions)?
        };

        let strict: Option<bool> = schema.get_as(intern!(py, "strict"))?;
        let allow_inf_nan: Option<bool> = schema.get_as(intern!(py, "allow_inf_nan"))?;

        if strict.is_some() | allow_inf_nan.is_some() {
            let config = PyDict::new_bound(py);
            if let Some(strict) = strict {
                config.set_item("strict", strict)?;
            }
            if let Some(allow_inf_nan) = allow_inf_nan {
                config.set_item("allow_inf_nan", allow_inf_nan)?;
            }
            validator = CombinedValidator::Config(ConfigValidator::try_new(config, Arc::new(validator))?);
        }

        Ok(validator)
    }
}

#[derive(Debug, Clone)]
pub struct FloatValidator;

impl BuildValidator for FloatValidator {
    const EXPECTED_TYPE: &'static str = "float";

    fn build(
        _schema: &Bound<'_, PyDict>,
        _config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        Ok(Self.into())
    }
}

impl_py_gc_traverse!(FloatValidator {});

impl Validator for FloatValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        let either_float = input.validate_float(state.strict_or(false))?.unpack(state);
        if !state.config.allow_inf_nan.unwrap_or(false) && !either_float.as_f64().is_finite() {
            return Err(ValError::new(ErrorTypeDefaults::FiniteNumber, input));
        }
        Ok(either_float.into_py(py))
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}

#[derive(Debug, Clone)]
pub struct ConstrainedFloatValidator {
    multiple_of: Option<f64>,
    le: Option<f64>,
    lt: Option<f64>,
    ge: Option<f64>,
    gt: Option<f64>,
}

impl_py_gc_traverse!(ConstrainedFloatValidator {});

impl Validator for ConstrainedFloatValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        let either_float = input.validate_float(state.strict_or(false))?.unpack(state);
        let float: f64 = either_float.as_f64();
        if !state.config.allow_inf_nan.unwrap_or(false) && !float.is_finite() {
            return Err(ValError::new(ErrorTypeDefaults::FiniteNumber, input));
        }
        if let Some(multiple_of) = self.multiple_of {
            let rem = float % multiple_of;
            let threshold = float.abs() / 1e9;
            if rem.abs() > threshold && (rem - multiple_of).abs() > threshold {
                return Err(ValError::new(
                    ErrorType::MultipleOf {
                        multiple_of: multiple_of.into(),
                        context: None,
                    },
                    input,
                ));
            }
        }
        if let Some(le) = self.le {
            if !matches!(float.partial_cmp(&le), Some(Ordering::Less | Ordering::Equal)) {
                return Err(ValError::new(
                    ErrorType::LessThanEqual {
                        le: le.into(),
                        context: None,
                    },
                    input,
                ));
            }
        }
        if let Some(lt) = self.lt {
            if !matches!(float.partial_cmp(&lt), Some(Ordering::Less)) {
                return Err(ValError::new(
                    ErrorType::LessThan {
                        lt: lt.into(),
                        context: None,
                    },
                    input,
                ));
            }
        }
        if let Some(ge) = self.ge {
            if !matches!(float.partial_cmp(&ge), Some(Ordering::Greater | Ordering::Equal)) {
                return Err(ValError::new(
                    ErrorType::GreaterThanEqual {
                        ge: ge.into(),
                        context: None,
                    },
                    input,
                ));
            }
        }
        if let Some(gt) = self.gt {
            if !matches!(float.partial_cmp(&gt), Some(Ordering::Greater)) {
                return Err(ValError::new(
                    ErrorType::GreaterThan {
                        gt: gt.into(),
                        context: None,
                    },
                    input,
                ));
            }
        }
        Ok(either_float.into_py(py))
    }

    fn get_name(&self) -> &str {
        "constrained-float"
    }
}

impl BuildValidator for ConstrainedFloatValidator {
    const EXPECTED_TYPE: &'static str = "float";
    fn build(
        schema: &Bound<'_, PyDict>,
        _config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        Ok(Self {
            multiple_of: schema.get_as(intern!(py, "multiple_of"))?,
            le: schema.get_as(intern!(py, "le"))?,
            lt: schema.get_as(intern!(py, "lt"))?,
            ge: schema.get_as(intern!(py, "ge"))?,
            gt: schema.get_as(intern!(py, "gt"))?,
        }
        .into())
    }
}
