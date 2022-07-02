use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::SchemaDict;
use crate::errors::{ErrorKind, ValError, ValResult};
use crate::input::Input;
use crate::recursion_guard::RecursionGuard;

use super::{BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

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
        guard_validate(self.validator_id, py, false, input, extra, slots, recursion_guard)
    }

    fn validate_strict<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        guard_validate(self.validator_id, py, true, input, extra, slots, recursion_guard)
    }

    fn get_name(&self, py: Python, slots: &[CombinedValidator]) -> String {
        // we just return the inner validator to make the recursive-container invisible in output messages
        let validator = unsafe { slots.get_unchecked(self.validator_id) };
        validator.get_name(py, slots)
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
        guard_validate(self.validator_id, py, false, input, extra, slots, recursion_guard)
    }

    fn validate_strict<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        guard_validate(self.validator_id, py, true, input, extra, slots, recursion_guard)
    }

    fn get_name(&self, _py: Python, _slots: &[CombinedValidator]) -> String {
        // we can't use the same logic as above because it can cause recursion,
        // this is a bodge until we improve names
        Self::EXPECTED_TYPE.to_string()
    }
}

fn guard_validate<'s, 'data>(
    validator_id: usize,
    py: Python<'data>,
    strict: bool,
    input: &'data impl Input<'data>,
    extra: &Extra,
    slots: &'data [CombinedValidator],
    recursion_guard: &'s mut RecursionGuard,
) -> ValResult<'data, PyObject> {
    if let Some(id) = input.identity() {
        if recursion_guard.contains_or_insert(id) {
            // we don't remove id here, we leave that to the validator which originally added id to `recursion_guard`
            Err(ValError::new(ErrorKind::RecursionLoop, input))
        } else {
            let output = validate(validator_id, py, strict, input, extra, slots, recursion_guard);
            recursion_guard.remove(&id);
            output
        }
    } else {
        validate(validator_id, py, strict, input, extra, slots, recursion_guard)
    }
}

fn validate<'s, 'data>(
    validator_id: usize,
    py: Python<'data>,
    strict: bool,
    input: &'data impl Input<'data>,
    extra: &Extra,
    slots: &'data [CombinedValidator],
    recursion_guard: &'s mut RecursionGuard,
) -> ValResult<'data, PyObject> {
    let validator = unsafe { slots.get_unchecked(validator_id) };
    if strict {
        validator.validate_strict(py, input, extra, slots, recursion_guard)
    } else {
        validator.validate(py, input, extra, slots, recursion_guard)
    }
}
