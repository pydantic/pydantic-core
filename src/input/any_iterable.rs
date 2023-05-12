use super::parse_json::JsonInput;
use pyo3::types::{PyDict, PyFrozenSet, PyIterator, PyList, PyMapping, PySequence, PySet, PyTuple};

#[cfg(not(PyPy))]
use pyo3::types::{PyDictItems, PyDictKeys, PyDictValues};

pub enum AnyIterable<'a> {
    List(&'a PyList),
    Tuple(&'a PyTuple),
    Set(&'a PySet),
    FrozenSet(&'a PyFrozenSet),
    Dict(&'a PyDict),
    #[cfg(not(PyPy))]
    DictKeys(&'a PyDictKeys),
    #[cfg(not(PyPy))]
    DictValues(&'a PyDictValues),
    #[cfg(not(PyPy))]
    DictItems(&'a PyDictItems),
    Mapping(&'a PyMapping),
    Sequence(&'a PySequence),
    Iterator(&'a PyIterator),
    JsonArray(&'a [JsonInput]),
}
