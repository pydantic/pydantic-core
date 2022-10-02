use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFrozenSet};

use crate::build_tools::SchemaDict;
use crate::errors::ValResult;
use crate::input::{GenericCollection, Input};
use crate::recursion_guard::RecursionGuard;

use super::list::generic_collection_build;
use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct FrozenSetValidator {
    strict: bool,
    item_validator: Option<Box<CombinedValidator>>,
    size_range: Option<(Option<usize>, Option<usize>)>,
    name: String,
}

impl BuildValidator for FrozenSetValidator {
    const EXPECTED_TYPE: &'static str = "frozenset";
    generic_collection_build!();
}

impl Validator for FrozenSetValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let seq = input.validate_frozenset(extra.strict.unwrap_or(self.strict))?;

        let output = match self.item_validator {
            Some(ref v) => seq.validate_to_vec(py, None, v, extra, slots, recursion_guard)?,
            None => match seq {
                GenericCollection::FrozenSet(f_set) => {
                    seq.check_len(self.size_range, input, f_set.len())?;
                    return Ok(f_set.into_py(py));
                }
                _ => seq.to_vec(py),
            },
        };

        let output_frozen = PyFrozenSet::new(py, &output)?;
        seq.check_len(self.size_range, input, output_frozen.len())?;
        Ok(output_frozen.into_py(py))
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn complete(&mut self, build_context: &BuildContext) -> PyResult<()> {
        match self.item_validator {
            Some(ref mut v) => v.complete(build_context),
            None => Ok(()),
        }
    }
}
