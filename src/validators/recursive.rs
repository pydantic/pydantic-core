use std::sync::{Arc, RwLock, Weak};

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_macros::dict_get_required;
use crate::build_macros::py_error;
use crate::errors::{as_internal, ValResult};
use crate::input::Input;

use super::{build_validator, Extra, Validator};

pub type ValidatorArc = Arc<RwLock<Box<dyn Validator>>>;

#[derive(Debug, Clone)]
pub struct RecursiveValidator {
    validator_arc: ValidatorArc,
}

impl RecursiveValidator {
    pub const EXPECTED_TYPE: &'static str = "recursive-container";
}

impl Validator for RecursiveValidator {
    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<Box<dyn Validator>> {
        let sub_schema = dict_get_required!(schema, "schema", &PyDict)?;
        let validator_box = build_validator(sub_schema, config)?;
        let validator_arc = Arc::new(RwLock::new(validator_box));
        {
            let mut validator_guard = validator_arc.write().unwrap();
            validator_guard.set_ref(&validator_arc);
        }
        Ok(Box::new(Self { validator_arc }))
    }

    fn set_ref(&mut self, _validator_arc: &ValidatorArc) {}

    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        extra: &Extra,
    ) -> ValResult<'data, PyObject> {
        let validator = self.validator_arc.read().unwrap();
        validator.validate(py, input, extra)
    }

    fn validate_strict<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        extra: &Extra,
    ) -> ValResult<'data, PyObject> {
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
    validator_ref: Option<Weak<RwLock<Box<dyn Validator>>>>,
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

    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        extra: &Extra,
    ) -> ValResult<'data, PyObject> {
        match self.validator_ref {
            Some(ref validator_ref) => {
                let validator_arc = validator_ref.upgrade().unwrap();
                let validator = validator_arc.read().unwrap();
                validator.validate(py, input, extra)
            }
            None => py_error!(PyRuntimeError; "ref not yet set on validator").map_err(as_internal),
        }
    }

    fn validate_strict<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        extra: &Extra,
    ) -> ValResult<'data, PyObject> {
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
