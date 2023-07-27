use num_bigint::BigInt;
use pyo3::{
    types::{PyDict, PyList},
    IntoPy, PyObject, Python, ToPyObject,
};

use crate::lazy_index_map::LazyIndexMap;

/// similar to serde `Value` but with int and float split
// FIXME remove clone, don't want to accidentally duplicate Vecs
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
    Object(Box<DataObject>),
    Py(PyObject),
}
pub type DataObject = LazyIndexMap<String, DataValue>;

impl ToPyObject for DataValue {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            Self::Null => py.None(),
            Self::Bool(b) => b.into_py(py),
            Self::Int(i) => i.into_py(py),
            Self::BigInt(b) => b.to_object(py),
            Self::Uint(i) => i.into_py(py),
            Self::Float(f) => f.into_py(py),
            Self::String(s) => s.into_py(py),
            Self::Array(v) => PyList::new(py, v.iter().map(|v| v.to_object(py))).into_py(py),
            Self::Object(o) => {
                let dict = PyDict::new(py);
                for (k, v) in o.iter() {
                    dict.set_item(k, v.to_object(py)).unwrap();
                }
                dict.into_py(py)
            }
            Self::Py(o) => o.clone_ref(py),
        }
    }
}

impl IntoPy<PyObject> for DataValue {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            Self::Null => py.None(),
            Self::Bool(b) => b.into_py(py),
            Self::Int(i) => i.into_py(py),
            Self::BigInt(b) => b.to_object(py),
            Self::Uint(i) => i.into_py(py),
            Self::Float(f) => f.into_py(py),
            Self::String(s) => s.into_py(py),
            Self::Array(v) => PyList::new(py, v.into_iter().map(|v| v.into_py(py))).into_py(py),
            Self::Object(o) => {
                let dict = PyDict::new(py);
                for (k, v) in *o {
                    dict.set_item(k, v.into_py(py)).unwrap();
                }
                dict.into_py(py)
            }
            Self::Py(o) => o.into_py(py),
        }
    }
}
