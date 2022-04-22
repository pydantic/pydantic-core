use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_macros::{dict_get, optional_dict_get};
use crate::errors::ValResult;
use crate::input::Input;

use super::{Extra, Validator};

#[derive(Debug, Clone)]
pub struct BoolValidator {
    strict: bool,
}

impl BoolValidator {
    pub const EXPECTED_TYPE: &'static str = "bool";
}

impl Validator for BoolValidator {
    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<Box<dyn Validator>> {
        let strict = match dict_get!(schema, "strict", bool) {
            Some(v) => v,
            None => optional_dict_get!(config, "strict", bool).unwrap_or(false),
        };
        Ok(Box::new(Self { strict }))
    }

    fn validate(&self, py: Python, input: &dyn Input, _extra: &Extra) -> ValResult<PyObject> {
        // TODO in theory this could be quicker if we used PyBool rather than going to a bool
        // and back again, might be worth profiling?
        let b = if self.strict {
            input.strict_bool(py)?
        } else {
            input.lax_bool(py)?
        };
        Ok(b.into_py(py))
    }

    fn clone_dyn(&self) -> Box<dyn Validator> {
        Box::new(self.clone())
    }
}
