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
        match validator_arc.write() {
            Ok(mut validator_guard) => validator_guard.set_ref(&validator_arc),
            Err(err) => return py_error!("Recursive container build error: {}", err),
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
        match self.validator_arc.read() {
            Ok(validator) => validator.validate(py, input, extra),
            Err(err) => {
                py_error!(PyRuntimeError; "Recursive container error: {}", err.to_string()).map_err(as_internal)
            }
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
        let error_msg: String = match self.validator_ref {
            Some(ref validator_ref) => {
                if let Some(validator_arc) = validator_ref.upgrade() {
                    match validator_arc.read() {
                        Ok(validator) => return validator.validate(py, input, extra),
                        Err(err) => format!("PoisonError: {}", err),
                    }
                } else {
                    "unable to upgrade weak reference".to_string()
                }
            }
            None => "ref not yet set".to_string(),
        };
        py_error!(PyRuntimeError; "Recursive reference error: {}", error_msg).map_err(as_internal)
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
