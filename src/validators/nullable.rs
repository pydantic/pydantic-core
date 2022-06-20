use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::SchemaDict;
use crate::input::Input;

use super::{BuildContext, BuildValidator, CombinedValidator, Extra, ValResult, Validator};

#[derive(Debug, Clone)]
pub struct NullableValidator {
    validator_id: usize,
}

impl BuildValidator for NullableValidator {
    const EXPECTED_TYPE: &'static str = "nullable";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let schema: &PyAny = schema.get_as_req("schema")?;
        let validator_id = build_context.add_unnamed_slot(schema, config)?;
        Ok(Self { validator_id }.into())
    }
}

impl Validator for NullableValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        match input.is_none() {
            true => Ok(py.None()),
            false => {
                let validator = unsafe { slots.get_unchecked(self.validator_id) };
                validator.validate(py, input, extra, slots)
            }
        }
    }

    fn validate_strict<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        match input.is_none() {
            true => Ok(py.None()),
            false => {
                let validator = unsafe { slots.get_unchecked(self.validator_id) };
                validator.validate_strict(py, input, extra, slots)
            }
        }
    }

    fn get_name<'data>(&self, _py: Python, _slots: &'data [CombinedValidator]) -> String {
        Self::EXPECTED_TYPE.to_string()
    }
}
