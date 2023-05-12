use super::parse_json::JsonInput;
use pyo3::types::{
    PyAny, PyDict, PyDictItems, PyDictKeys, PyDictValues, PyFrozenSet, PyIterator, PyList, PyMapping, PySequence,
    PySet, PyTuple,
};

pub enum AnyIterable<'a> {
    List(&'a PyList),
    Tuple(&'a PyTuple),
    Set(&'a PySet),
    FrozenSet(&'a PyFrozenSet),
    Dict(&'a PyDict),
    DictKeys(&'a PyDictKeys),
    DictValues(&'a PyDictValues),
    DictItems(&'a PyDictItems),
    Mapping(&'a PyMapping),
    Sequence(&'a PySequence),
    Iterator(&'a PyIterator),
    JsonArray(&'a [JsonInput]),
}
