use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::SchemaDict;
use crate::input::Input;

use super::{build_validator, BuildValidator, Extra, ValResult, ValidateEnum, Validator, SlotsBuilder, get_validator};

#[derive(Debug, Clone)]
pub struct OptionalValidator {
    validator_id: usize,
}

impl BuildValidator for OptionalValidator {
    const EXPECTED_TYPE: &'static str = "optional";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        slots_builder: &mut SlotsBuilder,
    ) -> PyResult<ValidateEnum> {
        let sub_schema: &PyAny = schema.get_as_req("schema")?;
        let validator = build_validator(sub_schema, config, slots_builder)?.0;
        let validator_id = slots_builder.add_anon(validator);
        Ok(Self { validator_id }.into())
    }
}

impl Validator for OptionalValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        extra: &Extra,
        slots: &'data [ValidateEnum],
    ) -> ValResult<'data, PyObject> {
        match input.is_none() {
            true => Ok(py.None()),
            false => {
                let validator = get_validator(slots, self.validator_id)?;
                validator.validate(py, input, extra, slots)
            },
        }
    }

    fn validate_strict<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        extra: &Extra,
        slots: &'data [ValidateEnum],
    ) -> ValResult<'data, PyObject> {
        match input.is_none() {
            true => Ok(py.None()),
            false => {
                let validator = get_validator(slots, self.validator_id)?;
                validator.validate_strict(py, input, extra, slots)
            },
        }
    }

    fn get_name(&self, _py: Python) -> String {
        Self::EXPECTED_TYPE.to_string()
    }
}
