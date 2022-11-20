use std::str::from_utf8;

use pyo3::exceptions::PyUnicodeEncodeError;
use pyo3::ffi::PyTypeObject;
use pyo3::once_cell::GILOnceCell;
use pyo3::prelude::*;
use pyo3::types::{
    PyByteArray, PyBytes, PyDate, PyDateTime, PyDelta, PyDict, PyFrozenSet, PyList, PySet, PyString, PyTime, PyTuple,
};

use crate::build_tools::py_err;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use strum_macros::EnumString;

use crate::url::{PyMultiHostUrl, PyUrl};

use super::shared::{py_err_se_err, BuildSerializer, CombinedSerializer, Extra, SerFormat, TypeSerializer};

#[derive(Debug, Clone)]
pub struct AnySerializer;

impl BuildSerializer for AnySerializer {
    const EXPECTED_TYPE: &'static str = "any";

    fn build(_schema: &PyDict, _config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        Ok(Self {}.into())
    }
}

impl TypeSerializer for AnySerializer {
    fn serde_serialize<S: Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        SerializeInfer::new(value, extra.ob_type_lookup).serialize(serializer)
    }
}

pub(super) fn fallback_to_python(value: &PyAny, extra: &Extra) -> PyResult<PyObject> {
    match extra.format {
        SerFormat::Json => fallback_to_python_json(value, extra.ob_type_lookup),
        _ => Ok(value.into_py(value.py())),
    }
}

pub(super) fn fallback_to_python_json(value: &PyAny, ob_type_lookup: &ObTypeLookup) -> PyResult<PyObject> {
    ob_type_to_python_json(&ob_type_lookup.get_type(value), value, ob_type_lookup)
}

pub(super) fn ob_type_to_python_json(
    ob_type: &ObType,
    value: &PyAny,
    ob_type_lookup: &ObTypeLookup,
) -> PyResult<PyObject> {
    let py = value.py();

    // have to do this to make sure subclasses of for example str are upcast to `str`
    macro_rules! extract_as {
        ($t:ty) => {{
            let v: $t = value.extract()?;
            Ok(v.into_py(py))
        }};
    }

    macro_rules! serialize_seq {
        ($t:ty) => {{
            let vec: Vec<PyObject> = value
                .cast_as::<$t>()?
                .iter()
                .map(|v| fallback_to_python_json(v, ob_type_lookup))
                .collect::<PyResult<_>>()?;
            Ok(PyList::new(py, vec).into_py(py))
        }};
    }

    match ob_type {
        ObType::Int => extract_as!(i64),
        // `bool` and `None` can't be subclasses, so no need to do the same on bool
        ObType::Float => extract_as!(f64),
        ObType::Str => extract_as!(&str),
        ObType::Bytes => {
            let py_bytes: &PyBytes = value.cast_as()?;
            match from_utf8(py_bytes.as_bytes()) {
                Ok(s) => Ok(s.into_py(py)),
                Err(e) => py_err!(PyUnicodeEncodeError; "{}", e),
            }
        }
        ObType::Bytearray => {
            let py_byte_array: &PyByteArray = value.cast_as()?;
            // see https://docs.rs/pyo3/latest/pyo3/types/struct.PyByteArray.html#method.as_bytes
            // for why this is marked unsafe
            let bytes = unsafe { py_byte_array.as_bytes() };
            match from_utf8(bytes) {
                Ok(s) => Ok(s.into_py(py)),
                Err(e) => py_err!(PyUnicodeEncodeError; "{}", e),
            }
        }
        // convert the tuple to a list, while recursively calling `fallback_to_python_json`
        ObType::Tuple => serialize_seq!(PyTuple),
        ObType::List => serialize_seq!(PyList),
        _ => Ok(value.into_py(value.py())),
    }
}

pub struct SerializeInfer<'py> {
    value: &'py PyAny,
    ob_type_lookup: &'py ObTypeLookup,
}

impl<'py> SerializeInfer<'py> {
    pub fn new(value: &'py PyAny, ob_type_lookup: &'py ObTypeLookup) -> Self {
        Self { value, ob_type_lookup }
    }
}

impl<'py> Serialize for SerializeInfer<'py> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        fallback_serialize(self.value, serializer, self.ob_type_lookup)
    }
}

pub fn fallback_serialize<S: Serializer>(
    value: &PyAny,
    serializer: S,
    ob_type_lookup: &ObTypeLookup,
) -> Result<S::Ok, S::Error> {
    fallback_serialize_known(&ob_type_lookup.get_type(value), value, serializer, ob_type_lookup)
}

pub fn fallback_serialize_known<S: Serializer>(
    ob_type: &ObType,
    value: &PyAny,
    serializer: S,
    ob_type_lookup: &ObTypeLookup,
) -> Result<S::Ok, S::Error> {
    macro_rules! serialize {
        ($t:ty) => {
            match value.extract::<$t>() {
                Ok(v) => v.serialize(serializer),
                Err(e) => Err(py_err_se_err(e)),
            }
        };
    }

    macro_rules! serialize_seq {
        ($t:ty) => {{
            let py_seq: $t = value.cast_as().map_err(py_err_se_err)?;
            let mut seq = serializer.serialize_seq(Some(py_seq.len()))?;
            for element in py_seq {
                seq.serialize_element(&SerializeInfer::new(element, ob_type_lookup))?
            }
            seq.end()
        }};
    }

    match ob_type {
        ObType::None => serializer.serialize_none(),
        ObType::Int => serialize!(i64),
        ObType::Bool => serialize!(bool),
        ObType::Float => serialize!(f64),
        ObType::Str => {
            let py_str: &PyString = value.cast_as().map_err(py_err_se_err)?;
            super::string::serialize_py_str(py_str, serializer)
        }
        ObType::Bytes => {
            let py_bytes: &PyBytes = value.cast_as().map_err(py_err_se_err)?;
            match from_utf8(py_bytes.as_bytes()) {
                Ok(s) => serializer.serialize_str(s),
                Err(e) => Err(py_err_se_err(e)),
            }
        }
        ObType::Bytearray => {
            let py_byte_array: &PyByteArray = value.cast_as().map_err(py_err_se_err)?;
            let bytes = unsafe { py_byte_array.as_bytes() };
            match from_utf8(bytes) {
                Ok(s) => serializer.serialize_str(s),
                Err(e) => Err(py_err_se_err(e)),
            }
        }
        ObType::Dict => {
            let py_dict: &PyDict = value.cast_as().map_err(py_err_se_err)?;

            let len = py_dict.len();
            let mut map = serializer.serialize_map(Some(len))?;
            for (k, v) in py_dict {
                map.serialize_entry(
                    &SerializeInfer::new(k, ob_type_lookup),
                    &SerializeInfer::new(v, ob_type_lookup),
                )?;
            }
            map.end()
        }
        ObType::List => serialize_seq!(&PyList),
        ObType::Tuple => serialize_seq!(&PyTuple),
        ObType::Set => serialize_seq!(&PySet),
        ObType::Frozenset => serialize_seq!(&PyFrozenSet),
        ObType::Datetime => {
            let dt_str = value
                .cast_as::<PyDateTime>()
                .map_err(py_err_se_err)?
                .str()
                .map_err(py_err_se_err)?
                .to_str()
                .map_err(py_err_se_err)?;
            if dt_str.ends_with("+00:00") {
                let mut is_dt = dt_str.to_string();
                is_dt.replace_range(dt_str.len() - 5.., "Z");
                serializer.serialize_str(&is_dt)
            } else {
                serializer.serialize_str(dt_str)
            }
        }
        ObType::Date => serializer.serialize_str(
            value
                .cast_as::<PyDate>()
                .map_err(py_err_se_err)?
                .str()
                .map_err(py_err_se_err)?
                .to_str()
                .map_err(py_err_se_err)?,
        ),
        ObType::Time => serializer.serialize_str(
            value
                .cast_as::<PyTime>()
                .map_err(py_err_se_err)?
                .str()
                .map_err(py_err_se_err)?
                .to_str()
                .map_err(py_err_se_err)?,
        ),
        _ => todo!(),
        // _ => serializer.serialize_none(),
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
    // mapping types
    dict: usize,
    // sequence types
    list: usize,
    tuple: usize,
    set: usize,
    frozenset: usize,
    // datetime types
    datetime: usize,
    date: usize,
    time: usize,
    timedelta: usize,
    // types from this package
    url: usize,
    multi_host_url: usize,
}

static TYPE_LOOKUP: GILOnceCell<ObTypeLookup> = GILOnceCell::new();

pub enum IsType {
    Exact,
    Subclass,
    False,
}

impl ObTypeLookup {
    fn new(py: Python) -> Self {
        let lib_url = url::Url::parse("https://example.com").unwrap();
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
            set: PySet::empty(py).unwrap().get_type_ptr() as usize,
            frozenset: PyFrozenSet::empty(py).unwrap().get_type_ptr() as usize,
            // mapping types
            dict: PyDict::new(py).get_type_ptr() as usize,
            // datetime types
            datetime: PyDateTime::new(py, 2000, 1, 1, 0, 0, 0, 0, None)
                .unwrap()
                .get_type_ptr() as usize,
            date: PyDate::new(py, 2000, 1, 1).unwrap().get_type_ptr() as usize,
            time: PyTime::new(py, 0, 0, 0, 0, None).unwrap().get_type_ptr() as usize,
            timedelta: PyDelta::new(py, 0, 0, 0, false).unwrap().get_type_ptr() as usize,
            // types from this package
            url: PyUrl::new(lib_url.clone()).into_py(py).as_ref(py).get_type_ptr() as usize,
            multi_host_url: PyMultiHostUrl::new(lib_url, None).into_py(py).as_ref(py).get_type_ptr() as usize,
        }
    }

    pub fn cached(py: Python<'_>) -> &Self {
        TYPE_LOOKUP.get_or_init(py, || Self::new(py))
    }

    pub fn is_type(&self, value: &PyAny, expected_ob_type: ObType) -> IsType {
        self.ob_type_is_expected(value.get_type_ptr(), expected_ob_type)
    }

    fn ob_type_is_expected(&self, type_ptr: *mut PyTypeObject, expected_ob_type: ObType) -> IsType {
        let ob_type = type_ptr as usize;
        let ans = match expected_ob_type {
            ObType::None => self.none == ob_type,
            ObType::Int => self.int == ob_type,
            ObType::Bool => self.bool == ob_type,
            ObType::Float => self.float == ob_type,
            ObType::Str => self.string == ob_type,
            ObType::Dict => self.dict == ob_type,
            ObType::List => self.list == ob_type,
            ObType::Tuple => self.tuple == ob_type,
            ObType::Set => self.set == ob_type,
            ObType::Frozenset => self.frozenset == ob_type,
            ObType::Bytes => self.bytes == ob_type,
            ObType::Datetime => self.datetime == ob_type,
            ObType::Date => self.date == ob_type,
            ObType::Time => self.time == ob_type,
            ObType::Timedelta => self.timedelta == ob_type,
            ObType::Bytearray => self.bytearray == ob_type,
            ObType::Url => self.url == ob_type,
            ObType::MultiHostUrl => self.multi_host_url == ob_type,
            ObType::Unknown => false,
        };

        if ans {
            IsType::Exact
        } else {
            // this allows for subtypes of the supported class types,
            // if we didn't successfully confirm the type, we try again with the next base type pointer provided
            // it's not null
            let base_type_ptr = unsafe { (*type_ptr).tp_base };
            if base_type_ptr.is_null() {
                IsType::False
            } else {
                match self.ob_type_is_expected(base_type_ptr, expected_ob_type) {
                    IsType::False => IsType::False,
                    _ => IsType::Subclass,
                }
            }
        }
    }

    pub fn get_type(&self, value: &PyAny) -> ObType {
        self.lookup_by_ob_type(value.get_type_ptr())
    }

    fn lookup_by_ob_type(&self, type_ptr: *mut PyTypeObject) -> ObType {
        let ob_type = type_ptr as usize;
        // this should be pretty fast, but still order is a bit important, so the most common types should come first
        // thus we don't follow the order of ObType
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
        } else if ob_type == self.dict {
            ObType::Dict
        } else if ob_type == self.list {
            ObType::List
        } else if ob_type == self.tuple {
            ObType::Tuple
        } else if ob_type == self.set {
            ObType::Set
        } else if ob_type == self.frozenset {
            ObType::Frozenset
        } else if ob_type == self.bytes {
            ObType::Bytes
        } else if ob_type == self.datetime {
            ObType::Datetime
        } else if ob_type == self.date {
            ObType::Date
        } else if ob_type == self.time {
            ObType::Time
        } else if ob_type == self.timedelta {
            ObType::Timedelta
        } else if ob_type == self.bytearray {
            ObType::Bytearray
        } else if ob_type == self.url {
            ObType::Url
        } else if ob_type == self.multi_host_url {
            ObType::MultiHostUrl
        } else {
            // this allows for subtypes of the supported class types,
            // if `ob_type` didn't match any member of self, we try again with the next base type pointer
            let base_type_ptr = unsafe { (*type_ptr).tp_base };
            if base_type_ptr.is_null() {
                ObType::Unknown
            } else {
                self.lookup_by_ob_type(base_type_ptr)
            }
        }
    }
}

#[derive(Debug, Clone, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum ObType {
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
    Set,
    Frozenset,
    // mapping types
    Dict,
    // datetime types
    Datetime,
    Date,
    Time,
    Timedelta,
    // types from this package
    Url,
    MultiHostUrl,
    // unknown type
    Unknown,
}
