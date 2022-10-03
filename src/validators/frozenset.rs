use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFrozenSet};

use crate::build_tools::SchemaDict;
use crate::errors::{ErrorKind, ValError, ValResult};
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

        let length = Some(seq.generic_len());

        let output = match self.item_validator {
            Some(ref v) => seq.validate_to_vec(py, length, v, extra, slots, recursion_guard)?,
            None => match seq {
                GenericCollection::FrozenSet(set) => {
                    seq.check_len(self.size_range, input)?;
                    return Ok(set.into_py(py));
                }
                _ => seq.to_vec(py),
            },
        };
        let final_set = PyFrozenSet::new(py, &output)?;
        if let Some((min_items, max_items)) = self.size_range {
            let input_length = final_set.len();
            if let Some(min_length) = min_items {
                if input_length < min_length {
                    return Err(ValError::new(
                        ErrorKind::TooShort {
                            min_length,
                            input_length,
                        },
                        input,
                    ));
                }
            }
            if let Some(max_length) = max_items {
                if input_length > max_length {
                    return Err(ValError::new(
                        ErrorKind::TooLong {
                            max_length,
                            input_length,
                        },
                        input,
                    ));
                }
            }
        }
        Ok(final_set.into_py(py))
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
