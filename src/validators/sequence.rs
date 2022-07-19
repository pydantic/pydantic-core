use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFrozenSet, PySet, PyTuple};

use crate::build_tools::SchemaDict;
use crate::errors::ValResult;
use crate::input::{GenericSequence, Input};
use crate::recursion_guard::RecursionGuard;

use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct SequenceValidator {
    item_validator: Option<Box<CombinedValidator>>,
    size_range: Option<(Option<usize>, Option<usize>)>,
    name: String,
}

impl BuildValidator for SequenceValidator {
    const EXPECTED_TYPE: &'static str = "sequence";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let item_validator = match schema.get_item(pyo3::intern!(py, "items_schema")) {
            Some(d) => Some(Box::new(build_validator(d, config, build_context)?.0)),
            None => None,
        };
        let name = match item_validator {
            Some(ref v) => format!("{}[{}]", Self::EXPECTED_TYPE, v.get_name()),
            None => format!("{}[any]", Self::EXPECTED_TYPE),
        };
        let min_items = schema.get_as(pyo3::intern!(py, "min_items"))?;
        let max_items = schema.get_as(pyo3::intern!(py, "max_items"))?;
        Ok(Self {
            item_validator,
            size_range: match min_items.is_some() || max_items.is_some() {
                true => Some((min_items, max_items)),
                false => None,
            },
            name,
        }
        .into())
    }
}

impl Validator for SequenceValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let seq = input.validate_list(false)?;

        let length = seq.check_len(self.size_range, input)?;

        match self.item_validator {
            Some(ref v) => {
                let vec = seq.validate_to_vec(py, length, v, extra, slots, recursion_guard)?;
                match seq {
                    GenericSequence::List(_) => Ok(vec.into_py(py)),
                    GenericSequence::Tuple(_) => Ok(PyTuple::new(py, &vec).into_py(py)),
                    GenericSequence::Set(_) => Ok(PySet::new(py, &vec)?.into_py(py)),
                    GenericSequence::FrozenSet(_) => Ok(PyFrozenSet::new(py, &vec)?.into_py(py)),
                    GenericSequence::JsonArray(_) => Ok(vec.to_object(py)),
                }
            }
            None => match seq {
                GenericSequence::List(list) => Ok(list.into_py(py)),
                GenericSequence::Tuple(tuple) => Ok(tuple.into_py(py)),
                GenericSequence::Set(set) => Ok(set.into_py(py)),
                GenericSequence::FrozenSet(f_set) => Ok(f_set.into_py(py)),
                GenericSequence::JsonArray(array) => Ok(array.to_object(py)),
            },
        }
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
