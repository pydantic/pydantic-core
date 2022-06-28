use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::SchemaDict;
use crate::errors::{err_val_error, ErrorKind, ValResult};
use crate::input::Input;

use super::{BuildContext, BuildValidator, CombinedValidator, Extra, RecursionGuard, Validator};

#[derive(Debug, Clone)]
pub struct RecursiveContainerValidator {
    validator_id: usize,
}

impl RecursiveContainerValidator {
    pub fn create(validator_id: usize) -> CombinedValidator {
        Self { validator_id }.into()
    }
}

impl Validator for RecursiveContainerValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let validator = unsafe { slots.get_unchecked(self.validator_id) };
        validator.validate(py, input, extra, slots, recursion_guard)
    }

    fn get_name(&self, _py: Python) -> String {
        "recursive-container".to_string()
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
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let name: String = schema.get_as_req("schema_ref")?;
        let validator_id = build_context.find_slot_id(&name)?;
        Ok(Self { validator_id }.into())
    }
}

impl Validator for RecursiveRefValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        if let Some(id) = input.identity() {
            eprintln!("---------------");
            dbg!(&recursion_guard, &input, &id);
            if recursion_guard.contains(&id) {
                return err_val_error!(kind = ErrorKind::RecursionLoop, input_value = input.as_error_value(),);
            }
            recursion_guard.insert(id);
        }
        let validator = unsafe { slots.get_unchecked(self.validator_id) };
        validator.validate(py, input, extra, slots, recursion_guard)
    }

    fn get_name(&self, _py: Python) -> String {
        Self::EXPECTED_TYPE.to_string()
    }
}
