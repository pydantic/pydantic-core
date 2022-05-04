use enum_dispatch::enum_dispatch;
use indexmap::map::Iter;

use pyo3::types::{PyAny, PyDict, PyFrozenSet, PyList, PySet, PyTuple};
use pyo3::{ffi, AsPyPointer};

use super::parse_json::{JsonArray, JsonInput, JsonObject};
use super::Input;

#[enum_dispatch]
pub enum GenericSequence<'data> {
    List(&'data PyList),
    Tuple(&'data PyTuple),
    Set(&'data PySet),
    FrozenSet(&'data PyFrozenSet),
    JsonArray(&'data JsonArray),
}

#[enum_dispatch(GenericSequence)]
pub trait SequenceLenIter<'data> {
    fn generic_len(&self) -> usize;

    fn generic_iter(&self) -> GenericSequenceIter<'data>;
}

impl<'data> SequenceLenIter<'data> for &'data PyList {
    fn generic_len(&self) -> usize {
        self.len()
    }

    fn generic_iter(&self) -> GenericSequenceIter<'data> {
        GenericSequenceIter::List(PyListIterator {
            sequence: self,
            index: 0,
        })
    }
}

impl<'data> SequenceLenIter<'data> for &'data PyTuple {
    fn generic_len(&self) -> usize {
        self.len()
    }

    fn generic_iter(&self) -> GenericSequenceIter<'data> {
        GenericSequenceIter::Tuple(PyTupleIterator {
            sequence: self,
            index: 0,
            length: self.len(),
        })
    }
}

impl<'data> SequenceLenIter<'data> for &'data PySet {
    fn generic_len(&self) -> usize {
        self.len()
    }

    fn generic_iter(&self) -> GenericSequenceIter<'data> {
        GenericSequenceIter::Set(PySetIterator {
            sequence: self,
            index: 0,
        })
    }
}

impl<'data> SequenceLenIter<'data> for &'data PyFrozenSet {
    fn generic_len(&self) -> usize {
        self.len()
    }

    fn generic_iter(&self) -> GenericSequenceIter<'data> {
        GenericSequenceIter::FrozenSet(PyFrozenSetIterator {
            sequence: self,
            index: 0,
        })
    }
}

impl<'data> SequenceLenIter<'data> for &'data JsonArray {
    fn generic_len(&self) -> usize {
        self.len()
    }

    fn generic_iter(&self) -> GenericSequenceIter<'data> {
        GenericSequenceIter::JsonArray(JsonArrayIterator {
            sequence: self,
            index: 0,
        })
    }
}

#[enum_dispatch]
pub enum GenericSequenceIter<'data> {
    List(PyListIterator<'data>),
    Tuple(PyTupleIterator<'data>),
    Set(PySetIterator<'data>),
    FrozenSet(PyFrozenSetIterator<'data>),
    JsonArray(JsonArrayIterator<'data>),
}

#[enum_dispatch(GenericSequenceIter)]
pub trait SequenceNext<'data> {
    fn _next(&mut self) -> Option<&'data dyn Input>;
}

impl<'data> Iterator for GenericSequenceIter<'data> {
    type Item = &'data dyn Input;

    #[inline]
    fn next(&mut self) -> Option<&'data dyn Input> {
        self._next()
    }
}

pub struct PyListIterator<'data> {
    sequence: &'data PyList,
    index: usize,
}

impl<'data> SequenceNext<'data> for PyListIterator<'data> {
    #[inline]
    fn _next(&mut self) -> Option<&'data dyn Input> {
        if self.index < self.sequence.len() {
            let item = unsafe { self.sequence.get_item_unchecked(self.index) };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

pub struct PyTupleIterator<'data> {
    sequence: &'data PyTuple,
    index: usize,
    length: usize,
}

impl<'data> SequenceNext<'data> for PyTupleIterator<'data> {
    #[inline]
    fn _next(&mut self) -> Option<&'data dyn Input> {
        if self.index < self.length {
            let item = unsafe { self.sequence.get_item_unchecked(self.index) };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

pub struct PySetIterator<'data> {
    sequence: &'data PySet,
    index: isize,
}

impl<'data> SequenceNext<'data> for PySetIterator<'data> {
    #[inline]
    fn _next(&mut self) -> Option<&'data dyn Input> {
        unsafe {
            let mut key: *mut ffi::PyObject = std::ptr::null_mut();
            let mut hash: ffi::Py_hash_t = 0;
            if ffi::_PySet_NextEntry(self.sequence.as_ptr(), &mut self.index, &mut key, &mut hash) != 0 {
                // _PySet_NextEntry returns borrowed object; for safety must make owned (see #890)
                let item: &PyAny = self.sequence.py().from_owned_ptr(ffi::_Py_NewRef(key));
                Some(item)
            } else {
                None
            }
        }
    }
}

pub struct PyFrozenSetIterator<'data> {
    sequence: &'data PyFrozenSet,
    index: isize,
}

impl<'data> SequenceNext<'data> for PyFrozenSetIterator<'data> {
    #[inline]
    fn _next(&mut self) -> Option<&'data dyn Input> {
        unsafe {
            let mut key: *mut ffi::PyObject = std::ptr::null_mut();
            let mut hash: ffi::Py_hash_t = 0;
            if ffi::_PySet_NextEntry(self.sequence.as_ptr(), &mut self.index, &mut key, &mut hash) != 0 {
                // _PySet_NextEntry returns borrowed object; for safety must make owned (see #890)
                let item: &PyAny = self.sequence.py().from_owned_ptr(ffi::_Py_NewRef(key));
                Some(item)
            } else {
                None
            }
        }
    }
}

pub struct JsonArrayIterator<'data> {
    sequence: &'data JsonArray,
    index: usize,
}

impl<'data> SequenceNext<'data> for JsonArrayIterator<'data> {
    #[inline]
    fn _next(&mut self) -> Option<&'data dyn Input> {
        match self.sequence.get(self.index) {
            Some(item) => {
                self.index += 1;
                Some(item)
            }
            None => None,
        }
    }
}

#[enum_dispatch]
pub enum GenericMapping<'data> {
    PyDict(&'data PyDict),
    JsonObject(&'data JsonObject),
}

// TODO work out how to avoid recursive error - should be `len`, `get` and `iter`
#[enum_dispatch(GenericMapping)]
pub trait MappingLenIter<'data> {
    fn generic_len(&self) -> usize;

    fn generic_get(&self, key: &str) -> Option<&'data dyn Input>;

    fn generic_iter(&self) -> GenericMappingIter<'data>;
}

impl<'data> MappingLenIter<'data> for &'data PyDict {
    fn generic_len(&self) -> usize {
        self.len()
    }

    fn generic_get(&self, key: &str) -> Option<&'data dyn Input> {
        self.get_item(key).map(|v| v as &dyn Input)
    }

    fn generic_iter(&self) -> GenericMappingIter<'data> {
        GenericMappingIter::PyDict(PyDictIterator { dict: self, index: 0 })
    }
}

impl<'data> MappingLenIter<'data> for &'data JsonObject {
    fn generic_len(&self) -> usize {
        self.len()
    }

    fn generic_get(&self, key: &str) -> Option<&'data dyn Input> {
        self.get(key).map(|v| v as &dyn Input)
    }

    fn generic_iter(&self) -> GenericMappingIter<'data> {
        GenericMappingIter::JsonObject(JsonObjectIterator { iter: self.iter() })
    }
}

#[enum_dispatch]
pub enum GenericMappingIter<'data> {
    PyDict(PyDictIterator<'data>),
    JsonObject(JsonObjectIterator<'data>),
}

/// helper trait implemented by all types in GenericMappingIter which is used when for the shared implementation of
/// `Iterator` for `GenericMappingIter`
#[enum_dispatch(GenericMappingIter)]
pub trait DictNext<'data> {
    fn _next(&mut self) -> Option<(&'data dyn Input, &'data dyn Input)>;
}

impl<'data> Iterator for GenericMappingIter<'data> {
    type Item = (&'data dyn Input, &'data dyn Input);

    #[inline]
    fn next(&mut self) -> Option<(&'data dyn Input, &'data dyn Input)> {
        self._next()
    }
}

pub struct PyDictIterator<'data> {
    dict: &'data PyDict,
    index: isize,
}

impl<'data> DictNext<'data> for PyDictIterator<'data> {
    #[inline]
    fn _next(&mut self) -> Option<(&'data dyn Input, &'data dyn Input)> {
        unsafe {
            let mut key: *mut ffi::PyObject = std::ptr::null_mut();
            let mut value: *mut ffi::PyObject = std::ptr::null_mut();
            if ffi::PyDict_Next(self.dict.as_ptr(), &mut self.index, &mut key, &mut value) != 0 {
                let py = self.dict.py();
                // PyDict_Next returns borrowed values; for safety must make them owned (see #890)
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

impl<'data> DictNext<'data> for JsonObjectIterator<'data> {
    #[inline]
    fn _next(&mut self) -> Option<(&'data dyn Input, &'data dyn Input)> {
        self.iter.next().map(|(k, v)| (k as &dyn Input, v as &dyn Input))
    }
}
