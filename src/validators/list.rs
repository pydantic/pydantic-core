use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::AsPyPointer;
use std::cmp::Ordering;

use crate::build_tools::SchemaDict;
use crate::errors::{ValError, ValLineError, ValResult};
use crate::input::{GenericCollection, Input};
use crate::recursion_guard::RecursionGuard;

use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct ListValidator {
    strict: bool,
    allow_any_iter: bool,
    item_validator: Option<Box<CombinedValidator>>,
    min_length: Option<usize>,
    max_length: Option<usize>,
    name: String,
}

pub fn get_items_schema(
    schema: &PyDict,
    config: Option<&PyDict>,
    build_context: &mut BuildContext<CombinedValidator>,
) -> PyResult<Option<Box<CombinedValidator>>> {
    match schema.get_item(pyo3::intern!(schema.py(), "items_schema")) {
        Some(d) => {
            let validator = build_validator(d, config, build_context)?;
            match validator {
                CombinedValidator::Any(_) => Ok(None),
                _ => Ok(Some(Box::new(validator))),
            }
        }
        None => Ok(None),
    }
}

macro_rules! length_check {
    ($input:ident, $field_type:literal, $min_length:expr, $max_length:expr, $obj:ident) => {{
        let mut op_actual_length: Option<usize> = None;
        if let Some(min_length) = $min_length {
            let actual_length = $obj.len();
            if actual_length < min_length {
                return Err(crate::errors::ValError::new(
                    crate::errors::ErrorType::TooShort {
                        field_type: $field_type.to_string(),
                        min_length,
                        actual_length,
                    },
                    $input,
                ));
            }
            op_actual_length = Some(actual_length);
        }
        if let Some(max_length) = $max_length {
            let actual_length = op_actual_length.unwrap_or_else(|| $obj.len());
            if actual_length > max_length {
                return Err(crate::errors::ValError::new(
                    crate::errors::ErrorType::TooLong {
                        field_type: $field_type.to_string(),
                        max_length,
                        actual_length,
                    },
                    $input,
                ));
            }
        }
    }};
}
pub(crate) use length_check;

impl BuildValidator for ListValidator {
    const EXPECTED_TYPE: &'static str = "list";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let item_validator = get_items_schema(schema, config, build_context)?;
        let inner_name = item_validator.as_ref().map(|v| v.get_name()).unwrap_or("any");
        let name = format!("{}[{inner_name}]", Self::EXPECTED_TYPE);
        Ok(Self {
            strict: crate::build_tools::is_strict(schema, config)?,
            allow_any_iter: schema.get_as(pyo3::intern!(py, "allow_any_iter"))?.unwrap_or(false),
            item_validator,
            min_length: schema.get_as(pyo3::intern!(py, "min_length"))?,
            max_length: schema.get_as(pyo3::intern!(py, "max_length"))?,
            name,
        }
        .into())
    }
}

struct PyListCreator {
    len: pyo3::ffi::Py_ssize_t,
    list: Py<PyList>,
    counter: pyo3::ffi::Py_ssize_t,
}

impl PyListCreator {
    fn with_capacity(py: Python, capacity: usize) -> PyResult<Self> {
        let len: pyo3::ffi::Py_ssize_t = capacity
            .try_into()
            .map_err(|_| PyValueError::new_err("list len out of range"))?;
        unsafe {
            let ptr = pyo3::ffi::PyList_New(len);
            let list: Py<PyList> = Py::from_owned_ptr(py, ptr);
            Ok(Self { len, list, counter: 0 })
        }
    }

    fn push(&mut self, item: PyObject) -> PyResult<()> {
        if self.counter == self.len {
            return Err(PyValueError::new_err("too many items added"));
        }
        let ptr = self.list.as_ptr();
        unsafe {
            #[cfg(not(Py_LIMITED_API))]
            pyo3::ffi::PyList_SET_ITEM(ptr, self.counter, item.into_ptr());
            #[cfg(Py_LIMITED_API)]
            pyo3::ffi::PyList_SetItem(ptr, self.counter, item.into_ptr());
        }
        self.counter += 1;
        Ok(())
    }

    fn complete(self, py: Python) -> PyResult<Py<PyList>> {
        match self.counter.cmp(&self.len) {
            Ordering::Equal => Ok(self.list),
            Ordering::Less => unsafe {
                let ptr = self.list.as_ptr();
                let slice_ptr = pyo3::ffi::PyList_GetSlice(ptr, 0, self.counter);
                Ok(Py::from_owned_ptr(py, slice_ptr))
            },
            Ordering::Greater => Err(PyValueError::new_err(format!(
                "list len mismatch {} != {}",
                self.counter, self.len
            ))),
        }
    }
}

impl Validator for ListValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let seq = input.validate_list(extra.strict.unwrap_or(self.strict), self.allow_any_iter)?;

        let output = match self.item_validator {
            Some(ref v) => match seq {
                GenericCollection::List(list) => {
                    let mut list_creator = PyListCreator::with_capacity(py, list.len())?;
                    let mut errors: Vec<ValLineError> = Vec::new();
                    for (index, item) in list.iter().enumerate() {
                        match v.validate(py, item, extra, slots, recursion_guard) {
                            Ok(item) => list_creator.push(item)?,
                            Err(ValError::LineErrors(line_errors)) => {
                                errors.extend(line_errors.into_iter().map(|err| err.with_outer_location(index.into())));
                            }
                            Err(ValError::Omit) => (),
                            Err(err) => return Err(err),
                        }
                    }

                    return if errors.is_empty() {
                        let list = list_creator.complete(py)?;
                        let list_ref = list.as_ref(py);
                        length_check!(input, "List", self.min_length, self.max_length, list_ref);
                        Ok(list.into_py(py))
                    } else {
                        Err(ValError::LineErrors(errors))
                    };
                }
                _ => seq.validate_to_vec(
                    py,
                    input,
                    self.max_length,
                    "List",
                    self.max_length,
                    v,
                    extra,
                    slots,
                    recursion_guard,
                )?,
            },
            None => match seq {
                GenericCollection::List(list) => {
                    length_check!(input, "List", self.min_length, self.max_length, list);
                    return Ok(list.into_py(py));
                }
                _ => seq.to_vec(py, input, "List", self.max_length)?,
            },
        };
        length_check!(input, "List", self.min_length, self.max_length, output);
        Ok(output.into_py(py))
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn complete(&mut self, build_context: &BuildContext<CombinedValidator>) -> PyResult<()> {
        match self.item_validator {
            Some(ref mut v) => v.complete(build_context),
            None => Ok(()),
        }
    }
}
