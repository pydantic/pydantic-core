use std::sync::{Arc, Mutex, Weak};

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_macros::dict_get_required;
use crate::errors::ValResult;
use crate::input::Input;

use super::{build_validator, Extra, Validator};

pub type ValidatorArc = Arc<Mutex<Box<dyn Validator>>>;

#[derive(Debug, Clone)]
pub struct RecursiveValidator {
    validator_arc: ValidatorArc,
}

impl RecursiveValidator {
    pub const EXPECTED_TYPE: &'static str = "recursive";
}

impl Validator for RecursiveValidator {
    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<Box<dyn Validator>> {
        let sub_schema = dict_get_required!(schema, "schema", &PyDict)?;
        let validator_box = build_validator(sub_schema, config)?;
        let validator_arc = Arc::new(Mutex::new(validator_box));
        {
            let mut validator_guard = validator_arc.lock().unwrap();
            validator_guard.set_ref(&validator_arc);
        }
        Ok(Box::new(Self { validator_arc }))
    }

    fn set_ref(&mut self, _validator_arc: &ValidatorArc) {}

    fn validate<'a>(&'a self, py: Python<'a>, input: &'a dyn Input, extra: &Extra) -> ValResult<'a, PyObject> {
        let validator = self.validator_arc.lock().unwrap();
        match validator.validate(py, input, extra) {
            Ok(value) => Ok(value),
            Err(err) => todo!("err: {}", err),
        }
    }

    fn validate_strict<'a>(&'a self, py: Python<'a>, input: &'a dyn Input, extra: &Extra) -> ValResult<'a, PyObject> {
        self.validate(py, input, extra)
    }

    fn get_name(&self, _py: Python) -> String {
        Self::EXPECTED_TYPE.to_string()
    }

    #[no_coverage]
    fn clone_dyn(&self) -> Box<dyn Validator> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct RecursiveRefValidator {
    validator_ref: Option<Weak<Mutex<Box<dyn Validator>>>>,
}

impl RecursiveRefValidator {
    pub const EXPECTED_TYPE: &'static str = "recursive-ref";
}

impl Validator for RecursiveRefValidator {
    fn build(_schema: &PyDict, _config: Option<&PyDict>) -> PyResult<Box<dyn Validator>> {
        Ok(Box::new(Self { validator_ref: None }))
    }

    fn set_ref(&mut self, validator_arc: &ValidatorArc) {
        self.validator_ref = Some(Arc::downgrade(validator_arc));
    }

    fn validate<'a>(&'a self, py: Python<'a>, input: &'a dyn Input, extra: &Extra) -> ValResult<'a, PyObject> {
        match self.validator_ref {
            Some(ref validator_ref) => {
                let validator_arc = validator_ref.upgrade().unwrap();
                let validator = validator_arc.lock().unwrap();
                match validator.validate(py, input, extra) {
                    Ok(value) => Ok(value),
                    Err(err) => todo!("err: {}", err),
                }
            }
            None => todo!(),
        }
    }

    fn validate_strict<'a>(&'a self, py: Python<'a>, input: &'a dyn Input, extra: &Extra) -> ValResult<'a, PyObject> {
        self.validate(py, input, extra)
    }

    fn get_name(&self, _py: Python) -> String {
        Self::EXPECTED_TYPE.to_string()
    }

    #[no_coverage]
    fn clone_dyn(&self) -> Box<dyn Validator> {
        Box::new(self.clone())
    }
}
