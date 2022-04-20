use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict};

use super::{Extra, Validator};
use crate::errors::ValResult;
use crate::input::Input;

#[derive(Debug, Clone)]
pub struct BoolValidator;

impl BoolValidator {
    pub const EXPECTED_TYPE: &'static str = "bool";
}

impl Validator for BoolValidator {
    fn build(_schema: &PyDict, _config: Option<&PyDict>) -> PyResult<Box<dyn Validator>> {
        Ok(Box::new(Self))
    }

    fn validate<'py>(&self, py: Python<'py>, input: &dyn Input, _extra: &Extra) -> ValResult<&'py PyAny> {
        // TODO in theory this could be quicker if we used PyBool rather than going to a bool
        // and back again, might be worth profiling?
        let b = PyBool::new(py, input.validate_bool(py)?);
        Ok(b.as_ref())
    }

    fn clone_dyn(&self) -> Box<dyn Validator> {
        Box::new(self.clone())
    }
}
