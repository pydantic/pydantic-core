use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFrozenSet, PyList, PySet, PyTuple};

use crate::errors::{LocItem, ValError, ValLineError, ValResult};
use crate::validators::{CombinedValidator, Extra, Validator};

use super::parse_json::{JsonArray, JsonObject};
use super::ToPy;

pub enum GenericSequence<'a> {
    List(&'a PyList),
    Tuple(&'a PyTuple),
    Set(&'a PySet),
    FrozenSet(&'a PyFrozenSet),
    JsonArray(&'a JsonArray),
}

macro_rules! sequence_derive_into {
    ($type:ty, $key:ident) => {
        impl<'a> From<&'a $type> for GenericSequence<'a> {
            fn from(s: &'a $type) -> GenericSequence<'a> {
                Self::$key(s)
            }
        }
    };
}
sequence_derive_into!(PyList, List);
sequence_derive_into!(PyTuple, Tuple);
sequence_derive_into!(PySet, Set);
sequence_derive_into!(PyFrozenSet, FrozenSet);
sequence_derive_into!(JsonArray, JsonArray);

impl<'a> GenericSequence<'a> {
    pub fn generic_len(&self) -> usize {
        match self {
            Self::List(v) => v.len(),
            Self::Tuple(v) => v.len(),
            Self::Set(v) => v.len(),
            Self::FrozenSet(v) => v.len(),
            Self::JsonArray(v) => v.len(),
        }
    }

    pub fn validate_to_vec<'s>(
        &self,
        py: Python<'a>,
        length: usize,
        validator: &'s CombinedValidator,
        extra: &Extra,
        slots: &'a [CombinedValidator],
    ) -> ValResult<'a, Vec<PyObject>> {
        let mut output: Vec<PyObject> = Vec::with_capacity(length);
        let mut errors: Vec<ValLineError> = Vec::new();
        macro_rules! iter {
            ($iterator:expr) => {
                for (index, item) in $iterator.enumerate() {
                    match validator.validate(py, item, extra, slots) {
                        Ok(item) => output.push(item),
                        Err(ValError::LineErrors(line_errors)) => {
                            let loc = vec![LocItem::I(index)];
                            errors.extend(line_errors.into_iter().map(|err| err.with_prefix_location(&loc)));
                        }
                        Err(err) => return Err(err),
                    }
                }
            };
        }

        match self {
            Self::List(sequence) => iter!(sequence.iter()),
            Self::Tuple(sequence) => iter!(sequence.iter()),
            Self::Set(sequence) => iter!(sequence.iter()),
            Self::FrozenSet(sequence) => iter!(sequence.iter()),
            Self::JsonArray(sequence) => iter!(sequence.iter()),
        }
        if errors.is_empty() {
            Ok(output)
        } else {
            Err(ValError::LineErrors(errors))
        }
    }

    pub fn copy_to_vec(&self, py: Python<'_>) -> Vec<PyObject> {
        macro_rules! to_vec {
            ($iterator:expr) => {
                $iterator.map(|item| item.into_py(py)).collect()
            };
        }
        match self {
            Self::List(sequence) => to_vec!(sequence.iter()),
            Self::Tuple(sequence) => to_vec!(sequence.iter()),
            Self::Set(sequence) => to_vec!(sequence.iter()),
            Self::FrozenSet(sequence) => to_vec!(sequence.iter()),
            Self::JsonArray(sequence) => sequence.iter().map(|item| item.to_py(py)).collect(),
        }
    }
}

pub enum GenericMapping<'a> {
    PyDict(&'a PyDict),
    JsonObject(&'a JsonObject),
}

impl<'a> From<&'a PyDict> for GenericMapping<'a> {
    fn from(d: &'a PyDict) -> GenericMapping<'a> {
        Self::PyDict(d)
    }
}

impl<'a> From<&'a JsonObject> for GenericMapping<'a> {
    fn from(d: &'a JsonObject) -> GenericMapping<'a> {
        Self::JsonObject(d)
    }
}

impl<'a> GenericMapping<'a> {
    pub fn generic_len(&self) -> usize {
        match self {
            Self::PyDict(d) => d.len(),
            Self::JsonObject(d) => d.len(),
        }
    }
}
