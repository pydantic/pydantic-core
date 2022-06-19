use enum_dispatch::enum_dispatch;
use indexmap::map::Iter;

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyFrozenSet, PyList, PySet, PyTuple};
use pyo3::{ffi, AsPyPointer};

use crate::errors::{LocItem, ValError, ValLineError, ValResult};
use crate::validators::{CombinedValidator, Extra, Validator};

use super::parse_json::{JsonArray, JsonInput, JsonObject};
use super::{Input, ToPy};

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
                GenericSequence::$key(s)
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
                for (index, item) in $iterator {
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
            Self::List(sequence) => iter!(PyListIterator::new(sequence)),
            Self::Tuple(sequence) => iter!(PyTupleIterator::new(sequence)),
            Self::Set(sequence) => iter!(PySetIterator::new(sequence)),
            Self::FrozenSet(sequence) => iter!(PySetIterator::new(sequence)),
            Self::JsonArray(sequence) => iter!(JsonArrayIterator::new(sequence)),
        }
        if errors.is_empty() {
            Ok(output)
        } else {
            Err(ValError::LineErrors(errors))
        }
    }

    pub fn validate_fixed_tuple<'s>(
        &self,
        py: Python<'a>,
        length: usize,
        items_validators: &'s [CombinedValidator],
        extra: &Extra,
        slots: &'a [CombinedValidator],
    ) -> ValResult<'a, Vec<PyObject>> {
        let mut output: Vec<PyObject> = Vec::with_capacity(length);
        let mut errors: Vec<ValLineError> = Vec::new();
        macro_rules! iter {
            ($iterator:expr) => {
                for (validator, (index, item)) in items_validators.iter().zip($iterator) {
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
            Self::List(sequence) => iter!(PyListIterator::new(sequence)),
            Self::Tuple(sequence) => iter!(PyTupleIterator::new(sequence)),
            Self::Set(sequence) => iter!(PySetIterator::new(sequence)),
            Self::FrozenSet(sequence) => iter!(PySetIterator::new(sequence)),
            Self::JsonArray(sequence) => iter!(JsonArrayIterator::new(sequence)),
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
                $iterator.map(|(_, item)| item.into_py(py)).collect()
            };
        }
        match self {
            Self::List(sequence) => to_vec!(PyListIterator::new(sequence)),
            Self::Tuple(sequence) => to_vec!(PyTupleIterator::new(sequence)),
            Self::Set(sequence) => to_vec!(PySetIterator::new(sequence)),
            Self::FrozenSet(sequence) => to_vec!(PySetIterator::new(sequence)),
            Self::JsonArray(sequence) => JsonArrayIterator::new(sequence)
                .map(|(_, item)| item.to_py(py))
                .collect(),
        }
    }
}

pub struct PyListIterator<'a> {
    sequence: &'a PyList,
    index: usize,
}

impl<'a> PyListIterator<'a> {
    fn new(sequence: &'a PyList) -> Self {
        Self { sequence, index: 0 }
    }
}

impl<'a> Iterator for PyListIterator<'a> {
    type Item = (usize, &'a PyAny);

    #[inline]
    fn next(&mut self) -> Option<(usize, &'a PyAny)> {
        if self.index < self.sequence.len() {
            let item = unsafe { self.sequence.get_item_unchecked(self.index) };
            let index = self.index;
            self.index += 1;
            Some((index, item))
        } else {
            None
        }
    }
}

pub struct PyTupleIterator<'a> {
    sequence: &'a PyTuple,
    index: usize,
    length: usize,
}

impl<'a> PyTupleIterator<'a> {
    fn new(sequence: &'a PyTuple) -> Self {
        Self {
            sequence,
            index: 0,
            length: sequence.len(),
        }
    }
}

impl<'a> Iterator for PyTupleIterator<'a> {
    type Item = (usize, &'a PyAny);

    fn next(&mut self) -> Option<(usize, &'a PyAny)> {
        if self.index < self.length {
            let item = unsafe { self.sequence.get_item_unchecked(self.index) };
            let index = self.index;
            self.index += 1;
            Some((index, item))
        } else {
            None
        }
    }
}

pub struct PySetIterator<'a> {
    sequence: &'a PyAny,
    index: isize,
}

impl<'a> PySetIterator<'a> {
    fn new(sequence: &'a PyAny) -> Self {
        Self { sequence, index: 0 }
    }
}

impl<'a> Iterator for PySetIterator<'a> {
    type Item = (usize, &'a PyAny);

    fn next(&mut self) -> Option<(usize, &'a PyAny)> {
        unsafe {
            let mut key: *mut ffi::PyObject = std::ptr::null_mut();
            let mut hash: ffi::Py_hash_t = 0;
            let index = self.index as usize;
            if ffi::_PySet_NextEntry(self.sequence.as_ptr(), &mut self.index, &mut key, &mut hash) != 0 {
                // _PySet_NextEntry returns borrowed object; for safety must make owned (see #890)
                let item: &PyAny = self.sequence.py().from_owned_ptr(ffi::_Py_NewRef(key));
                Some((index, item))
            } else {
                None
            }
        }
    }
}

pub struct JsonArrayIterator<'a> {
    sequence: &'a JsonArray,
    index: usize,
}

impl<'a> JsonArrayIterator<'a> {
    fn new(sequence: &'a JsonArray) -> Self {
        Self { sequence, index: 0 }
    }
}

impl<'a> Iterator for JsonArrayIterator<'a> {
    type Item = (usize, &'a JsonInput);

    fn next(&mut self) -> Option<(usize, &'a JsonInput)> {
        match self.sequence.get(self.index) {
            Some(item) => {
                let index = self.index;
                self.index += 1;
                Some((index, item))
            }
            None => None,
        }
    }
}

#[enum_dispatch]
pub enum GenericMapping<'a> {
    PyDict(&'a PyDict),
    JsonObject(&'a JsonObject),
}

// TODO work out how to avoid recursive error - should be `len`, `get` and `iter`
#[enum_dispatch(GenericMapping)]
pub trait MappingLenIter<'a> {
    fn generic_len(&self) -> usize;

    fn generic_get(&self, key: &str) -> Option<&'a dyn Input>;

    fn generic_iter(&self) -> GenericMappingIter<'a>;
}

impl<'a> MappingLenIter<'a> for &'a PyDict {
    #[inline]
    fn generic_len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn generic_get(&self, key: &str) -> Option<&'a dyn Input> {
        self.get_item(key).map(|v| v as &dyn Input)
    }

    #[inline]
    fn generic_iter(&self) -> GenericMappingIter<'a> {
        GenericMappingIter::PyDict(PyDictIterator { dict: self, index: 0 })
    }
}

impl<'a> MappingLenIter<'a> for &'a JsonObject {
    #[inline]
    fn generic_len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn generic_get(&self, key: &str) -> Option<&'a dyn Input> {
        self.get(key).map(|v| v as &dyn Input)
    }

    #[inline]
    fn generic_iter(&self) -> GenericMappingIter<'a> {
        GenericMappingIter::JsonObject(JsonObjectIterator { iter: self.iter() })
    }
}

#[enum_dispatch]
pub enum GenericMappingIter<'a> {
    PyDict(PyDictIterator<'a>),
    JsonObject(JsonObjectIterator<'a>),
}

/// helper trait implemented by all types in GenericMappingIter which is used when for the shared implementation of
/// `Iterator` for `GenericMappingIter`
#[enum_dispatch(GenericMappingIter)]
pub trait DictNext<'a> {
    fn _next(&mut self) -> Option<(&'a dyn Input, &'a dyn Input)>;
}

impl<'a> Iterator for GenericMappingIter<'a> {
    type Item = (&'a dyn Input, &'a dyn Input);

    #[inline]
    fn next(&mut self) -> Option<(&'a dyn Input, &'a dyn Input)> {
        self._next()
    }
}

pub struct PyDictIterator<'a> {
    dict: &'a PyDict,
    index: isize,
}

impl<'a> DictNext<'a> for PyDictIterator<'a> {
    #[inline]
    fn _next(&mut self) -> Option<(&'a dyn Input, &'a dyn Input)> {
        unsafe {
            let mut key: *mut ffi::PyObject = std::ptr::null_mut();
            let mut value: *mut ffi::PyObject = std::ptr::null_mut();
            if ffi::PyDict_Next(self.dict.as_ptr(), &mut self.index, &mut key, &mut value) != 0 {
                // PyDict_Next returns borrowed values; for safety must make them owned (see #890)
                let py = self.dict.py();
                let key: &PyAny = py.from_owned_ptr(ffi::_Py_NewRef(key));
                let value: &PyAny = py.from_owned_ptr(ffi::_Py_NewRef(value));
                Some((key, value))
            } else {
                None
            }
        }
    }
}

pub struct JsonObjectIterator<'a> {
    iter: Iter<'a, String, JsonInput>,
}

impl<'a> DictNext<'a> for JsonObjectIterator<'a> {
    #[inline]
    fn _next(&mut self) -> Option<(&'a dyn Input, &'a dyn Input)> {
        self.iter.next().map(|(k, v)| (k as &dyn Input, v as &dyn Input))
    }
}
