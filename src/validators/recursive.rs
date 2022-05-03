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

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        named_slots: &mut Vec<(Option<String>, Option<ValidateEnum>)>,
    ) -> PyResult<ValidateEnum> {
        let sub_schema: &PyAny = schema.get_as_req("schema")?;
        let name: String = schema.get_as_req("name")?;
        let validator_id = named_slots.len();
        named_slots.push((Some(name), None));
        let validator = build_validator(sub_schema, config, named_slots)?.0;
        named_slots[validator_id] = (None, Some(validator));
        Ok(Self { validator_id }.into())
    }
}

impl Validator for RecursiveValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        extra: &Extra,
        slots: &'data [ValidateEnum],
    ) -> ValResult<'data, PyObject> {
        match slots.get(self.validator_id) {
            Some(validator) => validator.validate(py, input, extra, slots),
            None => py_error!(PyRuntimeError; "Recursive container error").map_err(as_internal),
        }
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

    fn build(
        schema: &PyDict,
        _config: Option<&PyDict>,
        named_slots: &mut Vec<(Option<String>, Option<ValidateEnum>)>,
    ) -> PyResult<ValidateEnum> {
        let name: String = schema.get_as_req("name")?;
        let is_match = |(n, _): &(Option<String>, Option<ValidateEnum>)| match n {
            Some(n) => n == &name,
            None => false,
        };
        let validator_id = match named_slots.iter().position(is_match) {
            Some(id) => id,
            None => {
                return py_error!(PyRuntimeError; "Recursive reference error: ref '{}' not found", name);
            }
        };
        Ok(Self { validator_id }.into())
    }
}

impl Validator for RecursiveRefValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        extra: &Extra,
        slots: &'data [ValidateEnum],
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
