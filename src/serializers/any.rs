use pyo3::once_cell::GILOnceCell;
use pyo3::prelude::*;
use pyo3::types::{PyByteArray, PyBytes, PyDate, PyDateTime, PyDelta, PyDict, PyList, PyString, PyTime, PyTuple};

use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};

use super::{py_err_to_serde, BuildSerializer, CombinedSerializer, TypeSerializer};

#[derive(Debug, Clone)]
pub struct AnySerializer;

impl BuildSerializer for AnySerializer {
    const EXPECTED_TYPE: &'static str = "any";

    fn build(_schema: &PyDict, _config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        Ok(Self {}.into())
    }
}

impl TypeSerializer for AnySerializer {
    fn to_python(&self, py: Python, value: &PyAny, _format: Option<&str>) -> PyResult<PyObject> {
        Ok(value.into_py(py))
    }

    fn serde_serialize<S>(&self, value: &PyAny, serializer: S, ob_type_lookup: &ObTypeLookup) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = SerializeAny::new(value, ob_type_lookup);
        s.serialize(serializer)
    }
}

struct SerializeAny<'py> {
    obj: &'py PyAny,
    ob_type_lookup: &'py ObTypeLookup,
}

impl<'py> SerializeAny<'py> {
    pub fn new(obj: &'py PyAny, ob_type_lookup: &'py ObTypeLookup) -> Self {
        Self { obj, ob_type_lookup }
    }

    fn with_obj(&self, obj: &'py PyAny) -> Self {
        Self {
            obj,
            ob_type_lookup: self.ob_type_lookup,
        }
    }
}

impl<'py> Serialize for SerializeAny<'py> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        macro_rules! serialize {
            ($t:ty) => {
                match self.obj.extract::<$t>() {
                    Ok(v) => v.serialize(serializer),
                    Err(e) => Err(py_err_to_serde(e)),
                }
            };
        }

        match self.ob_type_lookup.get_type(self.obj) {
            ObType::None => serializer.serialize_none(),
            ObType::Int => serialize!(i64),
            ObType::Bool => serialize!(bool),
            ObType::Float => serialize!(f64),
            ObType::Str => serialize!(String),
            ObType::Bytes | ObType::Bytearray => serialize!(&[u8]),
            ObType::List => {
                let py_list: &PyList = self.obj.cast_as().map_err(py_err_to_serde)?;
                let mut seq = serializer.serialize_seq(Some(py_list.len()))?;
                for element in py_list {
                    seq.serialize_element(&self.with_obj(element))?
                }
                seq.end()
            }
            ObType::Tuple => {
                let py_list: &PyTuple = self.obj.cast_as().map_err(py_err_to_serde)?;
                let mut seq = serializer.serialize_seq(Some(py_list.len()))?;
                for element in py_list {
                    seq.serialize_element(&self.with_obj(element))?
                }
                seq.end()
            }
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObTypeLookup {
    none: usize,
    // numeric types
    int: usize,
    bool: usize,
    float: usize,
    // string types
    string: usize,
    bytes: usize,
    bytearray: usize,
    // sequence types
    list: usize,
    tuple: usize,
    // mapping types
    dict: usize,
    // datetime types
    datetime: usize,
    date: usize,
    time: usize,
    timedelta: usize,
}

static TYPE_LOOKUP: GILOnceCell<ObTypeLookup> = GILOnceCell::new();

impl ObTypeLookup {
    fn new(py: Python) -> Self {
        Self {
            none: py.None().as_ref(py).get_type_ptr() as usize,
            // numeric types
            int: 0i32.into_py(py).as_ref(py).get_type_ptr() as usize,
            bool: true.into_py(py).as_ref(py).get_type_ptr() as usize,
            float: 0f32.into_py(py).as_ref(py).get_type_ptr() as usize,
            // string types
            string: PyString::new(py, "s").get_type_ptr() as usize,
            bytes: PyBytes::new(py, b"s").get_type_ptr() as usize,
            bytearray: PyByteArray::new(py, b"s").get_type_ptr() as usize,
            // sequence types
            list: PyList::empty(py).get_type_ptr() as usize,
            tuple: PyTuple::empty(py).get_type_ptr() as usize,
            // mapping types
            dict: PyDict::new(py).get_type_ptr() as usize,
            // datetime types
            datetime: PyDateTime::new(py, 2000, 1, 1, 0, 0, 0, 0, None)
                .unwrap()
                .get_type_ptr() as usize,
            date: PyDate::new(py, 2000, 1, 1).unwrap().get_type_ptr() as usize,
            time: PyTime::new(py, 0, 0, 0, 0, None).unwrap().get_type_ptr() as usize,
            timedelta: PyDelta::new(py, 0, 0, 0, false).unwrap().get_type_ptr() as usize,
        }
    }

    pub fn cached(py: Python<'_>) -> &Self {
        TYPE_LOOKUP.get_or_init(py, || Self::new(py))
    }

    fn get_type(&self, obj: &PyAny) -> ObType {
        let ob_type = obj.get_type_ptr() as usize;
        if ob_type == self.none {
            ObType::None
        } else if ob_type == self.int {
            ObType::Int
        } else if ob_type == self.bool {
            ObType::Bool
        } else if ob_type == self.float {
            ObType::Float
        } else if ob_type == self.string {
            ObType::Str
        } else if ob_type == self.bytes {
            ObType::Bytes
        } else if ob_type == self.bytearray {
            ObType::Bytearray
        } else if ob_type == self.list {
            ObType::List
        } else if ob_type == self.tuple {
            ObType::Tuple
        } else if ob_type == self.dict {
            ObType::Dict
        } else if ob_type == self.datetime {
            ObType::Datetime
        } else if ob_type == self.date {
            ObType::Date
        } else if ob_type == self.time {
            ObType::Time
        } else if ob_type == self.timedelta {
            ObType::Timedelta
        } else {
            ObType::Unknown
        }
    }
}

#[cfg_attr(debug_assertions, derive(Debug))]
enum ObType {
    None,
    // numeric types
    Int,
    Bool,
    Float,
    // string types
    Str,
    Bytes,
    Bytearray,
    // sequence types
    List,
    Tuple,
    // mapping types
    Dict,
    // datetime types
    Datetime,
    Date,
    Time,
    Timedelta,
    // unknown type
    Unknown,
}
