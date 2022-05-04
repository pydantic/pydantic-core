use enum_dispatch::enum_dispatch;

use pyo3::types::{PyAny, PyDict, PyFrozenSet, PyList, PySet, PyTuple};
use pyo3::{ffi, AsPyPointer};

use super::parse_json::{JsonArray, JsonObject};
use super::{Input, ToPy};

pub enum ListInput<'data> {
    List(&'data PyList),
    Tuple(&'data PyTuple),
    Set(&'data PySet),
    FrozenSet(&'data PyFrozenSet),
    JsonArray(&'data JsonArray),
}

impl<'data> From<&'data PyList> for ListInput<'data> {
    fn from(sequence: &'data PyList) -> Self {
        Self::List(sequence)
    }
}

impl<'data> From<&'data PyTuple> for ListInput<'data> {
    fn from(sequence: &'data PyTuple) -> Self {
        Self::Tuple(sequence)
    }
}

impl<'data> From<&'data PySet> for ListInput<'data> {
    fn from(sequence: &'data PySet) -> Self {
        Self::Set(sequence)
    }
}

impl<'data> From<&'data PyFrozenSet> for ListInput<'data> {
    fn from(sequence: &'data PyFrozenSet) -> Self {
        Self::FrozenSet(sequence)
    }
}

impl<'data> From<&'data JsonArray> for ListInput<'data> {
    fn from(sequence: &'data JsonArray) -> Self {
        Self::JsonArray(sequence)
    }
}

impl<'data> ListInput<'data> {
    pub fn len(&self) -> usize {
        match self {
            Self::List(sequence) => sequence.len(),
            Self::Tuple(sequence) => sequence.len(),
            Self::Set(sequence) => sequence.len(),
            Self::FrozenSet(sequence) => sequence.len(),
            Self::JsonArray(sequence) => sequence.len(),
        }
    }

    pub fn iter(&self) -> GenericSequenceIter<'data> {
        match self {
            Self::List(sequence) => GenericSequenceIter::List(PyListIterator { sequence, index: 0 }),
            Self::Tuple(sequence) => GenericSequenceIter::Tuple(PyTupleIterator {
                sequence,
                index: 0,
                length: sequence.len(),
            }),
            Self::Set(sequence) => GenericSequenceIter::Set(PySetIterator { sequence, index: 0 }),
            Self::FrozenSet(sequence) => GenericSequenceIter::FrozenSet(PyFrozenSetIterator { sequence, index: 0 }),
            Self::JsonArray(sequence) => GenericSequenceIter::JsonArray(JsonArrayIterator { sequence, index: 0 }),
        }
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

// impl<'data> Iterator for JsonArrayIterator<'data> {
//     type Item = &'data dyn Input;
//
//     #[inline]
//     fn next(&mut self) -> Option<&'data dyn Input> {
//         match self.list.get(self.index) {
//             Some(item) => {
//                 self.index += 1;
//                 Some(item)
//             }
//             None => None,
//         }
//     }
//
//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         let len = self.list.len();
//         (
//             len.saturating_sub(self.index),
//             Some(len.saturating_sub(self.index)),
//         )
//     }
// }

// impl<'data> InputSequence for JsonArrayIterator<'data> {
//     #[inline]
//     fn next(&mut self) -> Option<&dyn Input> {
//         match self.list.get(self.index) {
//             Some(item) => {
//                 self.index += 1;
//                 Some(item)
//             }
//             None => None,
//         }
//     }
//
//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         let len = self.list.len();
//         (
//             len.saturating_sub(self.index),
//             Some(len.saturating_sub(self.index)),
//         )
//     }
//
//     fn input_len(&self) -> usize {
//         self.list.len()
//     }
// }

// impl<'data> ListIter<'data> for ListIterEnum<'data> {
//     fn input_iter(&self) -> Box<dyn Iterator<Item=&'data dyn Input> + 'data> {
//         match self {
//             ListIterEnum::List(list) => Box::new(list.iter().map(|item| item as &dyn Input)),
//             // ListIterEnum::Tuple(tuple) => Box::new(tuple.iter()),
//             // ListIterEnum::Set(set) => Box::new(set.iter()),
//             // ListIterEnum::FrozenSet(frozen_set) => Box::new(frozen_set.iter()),
//             ListIterEnum::JsonArray(json_array) => Box::new(json_array.iter().map(|item| item as &dyn Input)),
//         }
//     }
// }

// // these are ugly, is there any way to avoid the maps in iter, one of the boxes and/or the duplication?
// // is this harming performance, particularly the .map(|item| item)?
// // https://stackoverflow.com/a/47156134/949890
// pub trait ListInput<'data>: ToPy {
//     fn input_iter(&self) -> Box<dyn Iterator<Item = &'data dyn Input> + 'data>;
//
//     fn input_len(&self) -> usize;
// }
//
// impl<'data> ListInput<'data> for &'data PyList {
//     fn input_iter(&self) -> Box<dyn Iterator<Item = &'data dyn Input> + 'data> {
//         Box::new(self.iter().map(|item| item as &dyn Input))
//     }
//
//     fn input_len(&self) -> usize {
//         self.len()
//     }
// }
//
// impl<'data> ListInput<'data> for &'data PyTuple {
//     fn input_iter(&self) -> Box<dyn Iterator<Item = &'data dyn Input> + 'data> {
//         Box::new(self.iter().map(|item| item as &dyn Input))
//     }
//
//     fn input_len(&self) -> usize {
//         self.len()
//     }
// }
//
// impl<'data> ListInput<'data> for &'data PySet {
//     fn input_iter(&self) -> Box<dyn Iterator<Item = &'data dyn Input> + 'data> {
//         Box::new(self.iter().map(|item| item as &dyn Input))
//     }
//
//     fn input_len(&self) -> usize {
//         self.len()
//     }
// }
//
// impl<'data> ListInput<'data> for &'data PyFrozenSet {
//     fn input_iter(&self) -> Box<dyn Iterator<Item = &'data dyn Input> + 'data> {
//         Box::new(self.iter().map(|item| item as &dyn Input))
//     }
//
//     fn input_len(&self) -> usize {
//         self.len()
//     }
// }
//
// impl<'data> ListInput<'data> for &'data JsonArray {
//     fn input_iter(&self) -> Box<dyn Iterator<Item = &'data dyn Input> + 'data> {
//         Box::new(self.iter().map(|item| item as &dyn Input))
//     }
//
//     fn input_len(&self) -> usize {
//         self.len()
//     }
// }

///////////////////////

pub trait DictInput<'data>: ToPy {
    fn input_iter(&self) -> Box<dyn Iterator<Item = (&'data dyn Input, &'data dyn Input)> + 'data>;

    fn input_get(&self, key: &str) -> Option<&'data dyn Input>;

    fn input_len(&self) -> usize;
}

impl<'data> DictInput<'data> for &'data PyDict {
    fn input_iter(&self) -> Box<dyn Iterator<Item = (&'data dyn Input, &'data dyn Input)> + 'data> {
        Box::new(self.iter().map(|(k, v)| (k as &dyn Input, v as &dyn Input)))
    }

    fn input_get(&self, key: &str) -> Option<&'data dyn Input> {
        self.get_item(key).map(|item| item as &dyn Input)
    }

    fn input_len(&self) -> usize {
        self.len()
    }
}

impl<'data> DictInput<'data> for &'data JsonObject {
    fn input_iter(&self) -> Box<dyn Iterator<Item = (&'data dyn Input, &'data dyn Input)> + 'data> {
        Box::new(self.iter().map(|(k, v)| (k as &dyn Input, v as &dyn Input)))
    }

    fn input_get(&self, key: &str) -> Option<&'data dyn Input> {
        self.get(key).map(|item| item as &dyn Input)
    }

    fn input_len(&self) -> usize {
        self.len()
    }
}
