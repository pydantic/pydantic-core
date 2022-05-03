use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::{SchemaDict};
use crate::errors::{ValResult};
use crate::input::Input;

use super::{BuildValidator, Extra, ValidateEnum, Validator, SlotsBuilder, get_validator};

#[derive(Debug, Clone)]
pub struct RecursiveValidator {
    validator_id: usize,
}

impl BuildValidator for RecursiveValidator {
    const EXPECTED_TYPE: &'static str = "recursive-container";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        slots_builder: &mut SlotsBuilder,
    ) -> PyResult<ValidateEnum> {
        let sub_schema: &PyAny = schema.get_as_req("schema")?;
        let name: String = schema.get_as_req("name")?;
        let validator_id = slots_builder.build_add(name, sub_schema, config)?;
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
        let validator = get_validator(slots, self.validator_id)?;
        validator.validate(py, input, extra, slots)
    }

    fn get_name(&self, _py: Python) -> String {
        Self::EXPECTED_TYPE.to_string()
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
        slots_builder: &mut SlotsBuilder,
    ) -> PyResult<ValidateEnum> {
        let name: String = schema.get_as_req("name")?;
        let validator_id = slots_builder.find_id(&name)?;
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
        let validator = get_validator(slots, self.validator_id)?;
        validator.validate(py, input, extra, slots)
    }

    fn get_name(&self, _py: Python) -> String {
        Self::EXPECTED_TYPE.to_string()
    }
}
