use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::borrow::Cow;

use crate::errors::ValResult;
use crate::input::{GenericIterator, Input};
use crate::questions::Question;
use crate::recursion_guard::RecursionGuard;
use crate::ValidationError;

use super::list::get_items_schema;
use super::{BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct GeneratorValidator {
    item_validator: Option<Box<CombinedValidator>>,
    name: String,
}

impl BuildValidator for GeneratorValidator {
    const EXPECTED_TYPE: &'static str = "generator";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let item_validator = get_items_schema(schema, config, build_context)?;
        let name = match item_validator {
            Some(ref v) => format!("{}[{}]", Self::EXPECTED_TYPE, v.get_name()),
            None => format!("{}[any]", Self::EXPECTED_TYPE),
        };
        Ok(Self { item_validator, name }.into())
    }
}

impl Validator for GeneratorValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let iterator = input.validate_iter()?;
        let validator = self
            .item_validator
            .as_ref()
            .map(|v| InternalValidator::new(py, v, slots, extra, recursion_guard));

        let v_iterator = ValidatorIterator {
            iterator,
            validator,
            index: 0,
        };
        Ok(v_iterator.into_py(py))
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn ask(&self, question: &Question) -> bool {
        match self.item_validator {
            Some(ref v) => v.ask(question),
            None => false,
        }
    }

    fn complete(&mut self, build_context: &BuildContext) -> PyResult<()> {
        match self.item_validator {
            Some(ref mut v) => v.complete(build_context),
            None => Ok(()),
        }
    }
}

#[pyclass]
#[derive(Debug, Clone)]
struct ValidatorIterator {
    iterator: GenericIterator,
    validator: Option<InternalValidator>,
    // index is only used for python iterators, otherwise JsonIterator.index is used
    index: usize,
}

#[pymethods]
impl ValidatorIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>, py: Python) -> PyResult<Option<PyObject>> {
        let Self {
            validator, iterator, ..
        } = &mut *slf;
        match iterator {
            GenericIterator::PyIterator(ref mut iter) => match iter.as_ref(py).next() {
                Some(next_result) => {
                    let next = next_result?;
                    match validator {
                        Some(validator) => {
                            let r = validator.validate(py, next).map(Some);
                            slf.index += 1;
                            r
                        }
                        None => Ok(Some(next.to_object(py))),
                    }
                }
                None => Ok(None),
            },
            GenericIterator::JsonArray(ref mut iter) => match iter.next() {
                Some(next) => match validator {
                    Some(validator) => validator.validate(py, next).map(Some),
                    None => Ok(Some(next.to_object(py))),
                },
                None => Ok(None),
            },
        }
    }

    fn __repr__(&self) -> String {
        let schema = match self.validator {
            Some(ref v) => Cow::Owned(format!("{:?}", v.validator)),
            None => Cow::Borrowed("any"),
        };
        let index = match self.iterator {
            GenericIterator::PyIterator(_) => self.index,
            GenericIterator::JsonArray(ref iter) => iter.index(),
        };
        format!("ValidatorIterator(index={}, schema={})", index, schema)
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

/// Cloneable validator wrapper for use in generators in functions, this can be passed back to python
/// mid-validatoin
#[derive(Debug, Clone)]
pub struct InternalValidator {
    validator: CombinedValidator,
    slots: Vec<CombinedValidator>,
    // TODO, do we need data?
    data: Option<Py<PyDict>>,
    field: Option<String>,
    strict: Option<bool>,
    context: Option<PyObject>,
    recursion_guard: RecursionGuard,
}

impl InternalValidator {
    fn new(
        py: Python,
        validator: &CombinedValidator,
        slots: &[CombinedValidator],
        extra: &Extra,
        recursion_guard: &RecursionGuard,
    ) -> Self {
        Self {
            validator: validator.clone(),
            slots: slots.to_vec(),
            data: extra.data.map(|d| d.into_py(py)),
            field: extra.field.map(|f| f.to_string()),
            strict: extra.strict,
            context: extra.context.map(|d| d.into_py(py)),
            recursion_guard: recursion_guard.clone(),
        }
    }

    fn validate<'s, 'data>(&'s mut self, py: Python<'data>, input: &'data impl Input<'data>) -> PyResult<PyObject>
    where
        's: 'data,
    {
        let extra = Extra {
            data: self.data.as_ref().map(|data| data.as_ref(py)),
            field: self.field.as_deref(),
            strict: self.strict,
            context: self.context.as_ref().map(|data| data.as_ref(py)),
        };
        self.validator
            .validate(py, input, &extra, &self.slots, &mut self.recursion_guard)
            .map_err(|e| ValidationError::from_val_error(py, "ValidatorIterator".to_object(py), e))
    }
}
