use std::sync::{Arc, Weak};

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::{py_error, SchemaDict};
use crate::errors::{as_internal, ValResult};
use crate::input::Input;

use super::{build_validator, Extra, Validator};

pub type ValidatorArc = Arc<Box<dyn Validator>>;
pub type ValidatorWeak = Weak<Box<dyn Validator>>;

#[derive(Debug, Clone)]
pub struct RecursiveValidator {
    validator_arc: ValidatorArc,
    name: String,
}

impl RecursiveValidator {
    pub const EXPECTED_TYPE: &'static str = "recursive-container";
}

impl Validator for RecursiveValidator {
    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<Box<dyn Validator>> {
        let sub_schema: &PyAny = schema.get_as_req("schema")?;
        let validator = build_validator(sub_schema, config)?.0;
        let name: String = schema.get_as_req("name")?;

        let mut validator_arc = Arc::new(validator);
        let val_weak = Arc::downgrade(&validator_arc);

        unsafe {
            Arc::get_mut_unchecked(&mut validator_arc).set_ref(name.as_str(), val_weak)?;
        }
        Ok(Box::new(Self { validator_arc, name }))
    }

    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        extra: &Extra,
    ) -> ValResult<'data, PyObject> {
        self.validator_arc.validate(py, input, extra)
    }

    fn set_ref(&mut self, name: &str, validator_weak: ValidatorWeak) -> PyResult<()> {
        unsafe { Arc::get_mut_unchecked(&mut self.validator_arc).set_ref(name, validator_weak) }
    }

    fn get_name(&self, _py: Python) -> String {
        self.name.clone()
    }

    #[no_coverage]
    fn clone_dyn(&self) -> Box<dyn Validator> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct RecursiveRefValidator {
    validator_ref: Option<Weak<Box<dyn Validator>>>,
    name: String,
}

impl RecursiveRefValidator {
    pub const EXPECTED_TYPE: &'static str = "recursive-ref";
}

impl Validator for RecursiveRefValidator {
    fn build(schema: &PyDict, _config: Option<&PyDict>) -> PyResult<Box<dyn Validator>> {
        Ok(Box::new(Self {
            validator_ref: None,
            name: schema.get_as_req("name")?,
        }))
    }

    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        extra: &Extra,
    ) -> ValResult<'data, PyObject> {
        let error_msg: String = match self.validator_ref {
            Some(ref validator_ref) => match validator_ref.upgrade() {
                Some(validator_arc) => return validator_arc.validate(py, input, extra),
                None => "unable to upgrade weak reference".to_string(),
            },
            None => "ref not yet set".to_string(),
        };
        py_error!(PyRuntimeError; "Recursive reference error: {}", error_msg).map_err(as_internal)
    }

    fn set_ref(&mut self, name: &str, validator_weak: ValidatorWeak) -> PyResult<()> {
        if self.validator_ref.is_none() && name == self.name.as_str() {
            self.validator_ref = Some(validator_weak);
        }
        Ok(())
    }

    fn get_name(&self, _py: Python) -> String {
        self.name.clone()
    }

    #[no_coverage]
    fn clone_dyn(&self) -> Box<dyn Validator> {
        Box::new(self.clone())
    }
}
