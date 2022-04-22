use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_macros::{dict_get, optional_dict_get};
use crate::errors::ValResult;
use crate::input::Input;

use super::{Extra, Validator};

#[derive(Debug, Clone)]
pub struct BoolValidator;

impl BoolValidator {
    pub const EXPECTED_TYPE: &'static str = "bool";
}

impl Validator for BoolValidator {
    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<Box<dyn Validator>> {
        let strict = match dict_get!(schema, "strict", bool) {
            Some(v) => v,
            None => optional_dict_get!(config, "strict", bool).unwrap_or(false),
        };

        if strict {
            Ok(Box::new(StrictBoolValidator {}))
        } else {
            Ok(Box::new(Self {}))
        }
    }

    fn validate(&self, py: Python, input: &dyn Input, _extra: &Extra) -> ValResult<PyObject> {
        // TODO in theory this could be quicker if we used PyBool rather than going to a bool
        // and back again, might be worth profiling?
        Ok(input.lax_bool(py)?.into_py(py))
    }

    fn clone_dyn(&self) -> Box<dyn Validator> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct StrictBoolValidator;

impl Validator for StrictBoolValidator {
    fn build(_schema: &PyDict, _config: Option<&PyDict>) -> PyResult<Box<dyn Validator>> {
        unimplemented!("should be built by BoolValidator")
    }

    fn validate(&self, py: Python, input: &dyn Input, _extra: &Extra) -> ValResult<PyObject> {
        Ok(input.strict_bool(py)?.into_py(py))
    }

    fn clone_dyn(&self) -> Box<dyn Validator> {
        Box::new(self.clone())
    }
}
