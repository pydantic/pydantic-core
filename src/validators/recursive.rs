// use std::sync::{Arc, RwLock, Weak};

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::{py_error, SchemaDict};
use crate::errors::{as_internal, ValResult};
use crate::input::Input;

use super::{build_validator, BuildValidator, Extra, ValidateEnum, Validator};

pub type ValidatorArc = Box<ValidateEnum>;

#[derive(Debug, Clone)]
pub struct RecursiveValidator {
    validator_id: usize,
}

impl BuildValidator for RecursiveValidator {
    const EXPECTED_TYPE: &'static str = "recursive-container";

    fn build(schema: &PyDict, config: Option<&PyDict>, slots: &mut Vec<ValidateEnum>) -> PyResult<ValidateEnum> {
        let sub_schema: &PyAny = schema.get_as_req("schema")?;
        let slot_id = slots.len();
        let validator = build_validator(sub_schema, config, slots)?.0;
        slots.push(validator);
        Ok(Self { validator_id: slot_id }.into())
    }
}

impl Validator for RecursiveValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        extra: &Extra,
        slots: &'data Vec<ValidateEnum>,
    ) -> ValResult<'data, PyObject> {
        match slots.get(self.validator_id) {
            Some(validator) => validator.validate(py, input, extra, slots),
            None => py_error!(PyRuntimeError; "Recursive container error").map_err(as_internal),
        }
    }

    fn set_ref(&mut self, _name: &str, _validator_arc: &ValidatorArc) -> PyResult<()> {
        // match self.validator_arc.write() {
        //     Ok(mut validator_guard) => validator_guard.set_ref(name, validator_arc),
        //     Err(err) => py_error!("Recursive container set_ref error: {}", err),
        // }
        Ok(())
    }

    fn get_name(&self, _py: Python) -> String {
        Self::EXPECTED_TYPE.to_string()
        // self.name.clone()
    }
}

#[derive(Debug, Clone)]
pub struct RecursiveRefValidator {
    validator_id: usize,
}

impl BuildValidator for RecursiveRefValidator {
    const EXPECTED_TYPE: &'static str = "recursive-ref";

    fn build(_schema: &PyDict, _config: Option<&PyDict>, slots: &mut Vec<ValidateEnum>) -> PyResult<ValidateEnum> {
        Ok(Self {
            validator_id: slots.len(),
        }
        .into())
    }
}

impl Validator for RecursiveRefValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        extra: &Extra,
        slots: &'data Vec<ValidateEnum>,
    ) -> ValResult<'data, PyObject> {
        match slots.get(self.validator_id) {
            Some(validator) => validator.validate(py, input, extra, slots),
            None => py_error!(PyRuntimeError; "Recursive reference error: validator not found").map_err(as_internal),
        }
    }

    fn set_ref(&mut self, _name: &str, _validator_arc: &ValidatorArc) -> PyResult<()> {
        // if self.validator_ref.is_none() && name == self.name.as_str() {
        //     self.validator_ref = Some(Arc::downgrade(validator_arc));
        // }
        Ok(())
    }

    fn get_name(&self, _py: Python) -> String {
        Self::EXPECTED_TYPE.to_string()
        // self.name.clone()
    }
}
