use super::parse_json::JsonInput;
use pyo3::types::{PyDict, PyFrozenSet, PyIterator, PyList, PyMapping, PySequence, PySet, PyTuple};

pub enum AnyIterable<'a> {
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
    Sequence(&'a PySequence),
    Iterator(&'a PyIterator),
    JsonArray(&'a [JsonInput]),
}
