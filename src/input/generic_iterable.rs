use crate::errors::{py_err_string, ErrorType, ValError, ValResult};

use super::parse_json::{JsonInput, JsonObject};
use pyo3::{
    exceptions::PyTypeError,
    types::{
        PyByteArray, PyBytes, PyDict, PyFrozenSet, PyIterator, PyList, PyMapping, PySequence, PySet, PyString, PyTuple,
    },
    PyAny, PyErr, PyResult, Python, ToPyObject,
};

#[derive(Debug)]
pub enum GenericIterable<'a> {
    List(&'a PyList),
    Tuple(&'a PyTuple),
    Set(&'a PySet),
    FrozenSet(&'a PyFrozenSet),
    Dict(&'a PyDict),
    // Treat dict values / keys / items as generic iterators
    // since PyPy doesn't export the concrete types
    DictKeys(&'a PyIterator),
    DictValues(&'a PyIterator),
    DictItems(&'a PyIterator),
    Mapping(&'a PyMapping),
    String(&'a PyString),
    Bytes(&'a PyBytes),
    PyByteArray(&'a PyByteArray),
    Sequence(&'a PySequence),
    Iterator(&'a PyIterator),
    JsonArray(&'a [JsonInput]),
    JsonObject(&'a JsonObject),
}

type PyMappingItems<'a> = (&'a PyAny, &'a PyAny);

#[inline(always)]
fn extract_items(item: PyResult<&PyAny>) -> PyResult<PyMappingItems<'_>> {
    match item {
        Ok(v) => v.extract::<PyMappingItems>(),
        Err(e) => Err(e),
    }
}

#[inline(always)]
fn map_err<'data>(py: Python<'data>, err: PyErr, input: &'data PyAny) -> ValError<'data> {
    ValError::new(
        ErrorType::IterationError {
            error: py_err_string(py, err),
        },
        input,
    )
}

impl<'a, 'py: 'a> GenericIterable<'a> {
    pub fn len(&self) -> Option<usize> {
        match &self {
            GenericIterable::List(iter) => Some(iter.len()),
            GenericIterable::Tuple(iter) => Some(iter.len()),
            GenericIterable::Set(iter) => Some(iter.len()),
            GenericIterable::FrozenSet(iter) => Some(iter.len()),
            GenericIterable::Dict(iter) => Some(iter.len()),
            GenericIterable::DictKeys(iter) => iter.len().ok(),
            GenericIterable::DictValues(iter) => iter.len().ok(),
            GenericIterable::DictItems(iter) => iter.len().ok(),
            GenericIterable::Mapping(iter) => iter.len().ok(),
            GenericIterable::String(iter) => iter.len().ok(),
            GenericIterable::Bytes(iter) => iter.len().ok(),
            GenericIterable::PyByteArray(iter) => Some(iter.len()),
            GenericIterable::Sequence(iter) => iter.len().ok(),
            GenericIterable::Iterator(iter) => iter.len().ok(),
            GenericIterable::JsonArray(iter) => Some(iter.len()),
            GenericIterable::JsonObject(iter) => Some(iter.len()),
        }
    }
    pub fn into_sequence_iterator(
        self,
        py: Python<'py>,
    ) -> PyResult<Box<dyn Iterator<Item = PyResult<&'a PyAny>> + 'a>> {
        match self {
            GenericIterable::List(iter) => Ok(Box::new(iter.iter().map(Ok))),
            GenericIterable::Tuple(iter) => Ok(Box::new(iter.iter().map(Ok))),
            GenericIterable::Set(iter) => Ok(Box::new(iter.iter().map(Ok))),
            GenericIterable::FrozenSet(iter) => Ok(Box::new(iter.iter().map(Ok))),
            // Note that this iterates over only the keys, just like doing iter({}) in Python
            GenericIterable::Dict(iter) => Ok(Box::new(iter.iter().map(|(k, _)| Ok(k)))),
            GenericIterable::DictKeys(iter) => Ok(Box::new(iter)),
            GenericIterable::DictValues(iter) => Ok(Box::new(iter)),
            GenericIterable::DictItems(iter) => Ok(Box::new(iter)),
            // Note that this iterates over only the keys, just like doing iter({}) in Python
            GenericIterable::Mapping(iter) => Ok(Box::new(iter.keys()?.iter()?)),
            GenericIterable::String(iter) => Ok(Box::new(iter.iter()?)),
            GenericIterable::Bytes(iter) => Ok(Box::new(iter.iter()?)),
            GenericIterable::PyByteArray(iter) => Ok(Box::new(iter.iter()?)),
            GenericIterable::Sequence(iter) => Ok(Box::new(iter.iter()?)),
            GenericIterable::Iterator(iter) => Ok(Box::new(iter)),
            GenericIterable::JsonArray(iter) => Ok(Box::new(iter.iter().map(move |v| {
                let v = v.to_object(py);
                Ok(v.into_ref(py))
            }))),
            // Note that this iterates over only the keys, just like doing iter({}) in Python, just for consistency
            GenericIterable::JsonObject(iter) => Ok(Box::new(
                iter.iter().map(move |(k, _)| Ok(k.to_object(py).into_ref(py))),
            )),
        }
    }

    pub fn into_mapping_items_iterator(
        self,
        py: Python<'a>,
    ) -> PyResult<Box<dyn Iterator<Item = ValResult<'a, PyMappingItems<'a>>> + 'a>> {
        let py2 = py;
        match self {
            GenericIterable::List(iter) => {
                Ok(Box::new(iter.iter().map(move |v| {
                    extract_items(Ok(v)).map_err(|e| map_err(py2, e, iter.as_ref()))
                })))
            }
            GenericIterable::Tuple(iter) => {
                Ok(Box::new(iter.iter().map(move |v| {
                    extract_items(Ok(v)).map_err(|e| map_err(py2, e, iter.as_ref()))
                })))
            }
            GenericIterable::Set(iter) => {
                Ok(Box::new(iter.iter().map(move |v| {
                    extract_items(Ok(v)).map_err(|e| map_err(py2, e, iter.as_ref()))
                })))
            }
            GenericIterable::FrozenSet(iter) => {
                Ok(Box::new(iter.iter().map(move |v| {
                    extract_items(Ok(v)).map_err(|e| map_err(py2, e, iter.as_ref()))
                })))
            }
            // Note that we iterate over (key, value), unlike doing iter({}) in Python
            GenericIterable::Dict(iter) => Ok(Box::new(iter.iter().map(Ok))),
            // Keys or values can be tuples
            GenericIterable::DictKeys(iter) => Ok(Box::new(
                iter.map(extract_items)
                    .map(move |r| r.map_err(|e| map_err(py2, e, iter.as_ref()))),
            )),
            GenericIterable::DictValues(iter) => Ok(Box::new(
                iter.map(extract_items)
                    .map(move |r| r.map_err(|e| map_err(py2, e, iter.as_ref()))),
            )),
            GenericIterable::DictItems(iter) => Ok(Box::new(
                iter.map(extract_items)
                    .map(move |r| r.map_err(|e| map_err(py2, e, iter.as_ref()))),
            )),
            // Note that we iterate over (key, value), unlike doing iter({}) in Python
            GenericIterable::Mapping(iter) => Ok(Box::new(
                iter.items()?
                    .iter()?
                    .map(extract_items)
                    .map(move |r| r.map_err(|e| map_err(py2, e, iter.as_ref()))),
            )),
            // In Python if you do dict("foobar") you get "dictionary update sequence element #0 has length 1; 2 is required"
            // This is similar but arguably a better error message
            GenericIterable::String(_) => Err(PyTypeError::new_err(
                "Expected an iterable of (key, value) pairs, got a string",
            )),
            GenericIterable::Bytes(_) => Err(PyTypeError::new_err(
                "Expected an iterable of (key, value) pairs, got a bytes",
            )),
            GenericIterable::PyByteArray(_) => Err(PyTypeError::new_err(
                "Expected an iterable of (key, value) pairs, got a bytearray",
            )),
            // Obviously these may be things that are not convertible to a tuple of (Hashable, Any)
            // Python fails with a similar error message to above, ours will be slightly different (PyO3 will fail to extract) but similar enough
            GenericIterable::Sequence(iter) => Ok(Box::new(
                iter.iter()?
                    .map(extract_items)
                    .map(move |r| r.map_err(|e| map_err(py2, e, iter.as_ref()))),
            )),
            GenericIterable::Iterator(iter) => Ok(Box::new(
                iter.iter()?
                    .map(extract_items)
                    .map(move |r| r.map_err(|e| map_err(py2, e, iter.as_ref()))),
            )),
            GenericIterable::JsonArray(iter) => Ok(Box::new(
                iter.iter()
                    .map(move |v| extract_items(Ok(v.to_object(py).into_ref(py))))
                    .map(move |r| r.map_err(|e| map_err(py2, e, iter.to_object(py).into_ref(py)))),
            )),
            // Note that we iterate over (key, value), unlike doing iter({}) in Python
            GenericIterable::JsonObject(iter) => Ok(Box::new(iter.iter().map(move |(k, v)| {
                let k = PyString::new(py, k).as_ref();
                let v = v.to_object(py).into_ref(py);
                Ok((k, v))
            }))),
        }
    }
}
