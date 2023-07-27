use num_bigint::BigInt;
use pyo3::PyObject;

use crate::lazy_index_map::LazyIndexMap;

/// similar to serde `Value` but with int and float split
#[derive(Clone, Debug)]
pub enum DataValue {
    Null,
    Bool(bool),
    Int(i64),
    BigInt(BigInt),
    Uint(u64),
    Float(f64),
    String(String),
    Array(Vec<DataValue>),
    Object(DataObject),
    Py(PyObject),
}
pub type DataObject = LazyIndexMap<String, DataValue>;
