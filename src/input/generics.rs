use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFrozenSet, PyList, PySet, PyTuple};

use crate::errors::{ValError, ValLineError, ValResult};
use crate::validators::{CombinedValidator, Extra, Validator};

use super::input_abstract::Input;
use super::parse_json::{JsonArray, JsonObject};

pub enum GenericSequence<'a> {
    List(&'a PyList),
    Tuple(&'a PyTuple),
    Set(&'a PySet),
    FrozenSet(&'a PyFrozenSet),
    JsonArray(&'a JsonArray),
}

macro_rules! derive_from {
    ($enum:ident, $type:ty, $key:ident) => {
        impl<'a> From<&'a $type> for $enum<'a> {
            fn from(s: &'a $type) -> $enum<'a> {
                Self::$key(s)
            }
        }
    };
}
derive_from!(GenericSequence, PyList, List);
derive_from!(GenericSequence, PyTuple, Tuple);
derive_from!(GenericSequence, PySet, Set);
derive_from!(GenericSequence, PyFrozenSet, FrozenSet);
derive_from!(GenericSequence, JsonArray, JsonArray);

fn validate_generic_vec<'a, 's>(
    py: Python<'a>,
    iter: impl Iterator<Item = &'a (impl Input<'a> + 'a)>,
    length: usize,
    validator: &'s CombinedValidator,
    extra: &Extra,
    slots: &'a [CombinedValidator],
) -> ValResult<'a, Vec<PyObject>> {
    let mut output: Vec<PyObject> = Vec::with_capacity(length);
    let mut errors: Vec<ValLineError> = Vec::new();
    for (index, item) in iter.enumerate() {
        match validator.validate(py, item, extra, slots) {
            Ok(item) => output.push(item),
            Err(ValError::LineErrors(line_errors)) => {
                errors.extend(line_errors.into_iter().map(|err| err.with_outer_location(index.into())));
            }
            Err(err) => return Err(err),
        }
    }

    if errors.is_empty() {
        Ok(output)
    } else {
        Err(ValError::LineErrors(errors))
    }
}

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
        match self {
            Self::List(seq) => validate_generic_vec(py, seq.iter(), length, validator, extra, slots),
            Self::Tuple(seq) => validate_generic_vec(py, seq.iter(), length, validator, extra, slots),
            Self::Set(seq) => validate_generic_vec(py, seq.iter(), length, validator, extra, slots),
            Self::FrozenSet(seq) => validate_generic_vec(py, seq.iter(), length, validator, extra, slots),
            Self::JsonArray(seq) => validate_generic_vec(py, seq.iter(), length, validator, extra, slots),
        }
    }
}

pub enum GenericMapping<'a> {
    PyDict(&'a PyDict),
    JsonObject(&'a JsonObject),
}

derive_from!(GenericMapping, PyDict, PyDict);
derive_from!(GenericMapping, JsonObject, JsonObject);

impl<'a> GenericMapping<'a> {
    pub fn generic_len(&self) -> usize {
        match self {
            Self::PyDict(v) => v.len(),
            Self::JsonObject(v) => v.len(),
        }
    }
}
