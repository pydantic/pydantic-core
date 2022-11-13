use std::borrow::Cow;
use std::str::from_utf8;

use pyo3::exceptions::PyUnicodeEncodeError;
use pyo3::once_cell::GILOnceCell;
use pyo3::prelude::*;
use pyo3::types::{
    PyByteArray, PyBytes, PyDate, PyDateTime, PyDelta, PyDict, PyFrozenSet, PyList, PySet, PyString, PyTime, PyTuple,
};

use crate::build_tools::py_err;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use strum_macros::EnumString;

use crate::url::{PyMultiHostUrl, PyUrl};

use super::{py_err_se_err, BuildSerializer, CombinedSerializer, TypeSerializer};

#[derive(Debug, Clone)]
pub struct AnySerializer;

impl BuildSerializer for AnySerializer {
    const EXPECTED_TYPE: &'static str = "any";

    fn build_combined(_schema: &PyDict, _config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        Ok(Self {}.into())
    }
}

impl TypeSerializer for AnySerializer {
    // TODO implement to_python and respect include and exclude on list, tuple and dict

    fn to_python_json(
        &self,
        value: &PyAny,
        ob_type_lookup: &ObTypeLookup,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
    ) -> PyResult<PyObject> {
        let py = value.py();
        let ob_type = ob_type_lookup.get_type(value);
        match ob_type {
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
            ObType::Tuple => {
                let include = Cow::Owned(super::list_tuple::to_inc_ex(include)?);
                let exclude = Cow::Owned(super::list_tuple::to_inc_ex(exclude)?);
                // is this really the best way to do this?
                let item_serializer = self.clone().into();
                super::list_tuple::tuple_to_python_json(value, include, exclude, &item_serializer)
            }
            _ => self.to_python(value, Some("json"), include, exclude),
        }
    }

    fn serde_serialize<S: Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        ob_type_lookup: &ObTypeLookup,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
    ) -> Result<S::Ok, S::Error> {
        SerializeInfer::new(value, ob_type_lookup).serialize(serializer)
    }
}

pub struct SerializeInfer<'py> {
    obj: &'py PyAny,
    ob_type_lookup: &'py ObTypeLookup,
}

impl<'py> SerializeInfer<'py> {
    pub fn new(obj: &'py PyAny, ob_type_lookup: &'py ObTypeLookup) -> Self {
        Self { obj, ob_type_lookup }
    }
}

impl<'py> Serialize for SerializeInfer<'py> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        common_serialize(
            self.obj,
            &self.ob_type_lookup.get_type(self.obj),
            serializer,
            self.ob_type_lookup,
        )
    }
}

pub fn common_serialize<S: Serializer>(
    obj: &PyAny,
    ob_type: &ObType,
    serializer: S,
    ob_type_lookup: &ObTypeLookup,
) -> Result<S::Ok, S::Error> {
    macro_rules! serialize {
        ($t:ty) => {
            match obj.extract::<$t>() {
                Ok(v) => v.serialize(serializer),
                Err(e) => Err(py_err_se_err(e)),
            }
        };
    }

    macro_rules! serialize_seq {
        ($t:ty) => {{
            let py_seq: $t = obj.cast_as().map_err(py_err_se_err)?;
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
        ObType::Str => super::string::serialize_str(obj, serializer),
        ObType::Bytes => {
            let py_bytes: &PyBytes = obj.cast_as().map_err(py_err_se_err)?;
            match from_utf8(py_bytes.as_bytes()) {
                Ok(s) => serializer.serialize_str(s),
                Err(e) => Err(py_err_se_err(e)),
            }
        }
        ObType::Bytearray => {
            let py_byte_array: &PyByteArray = obj.cast_as().map_err(py_err_se_err)?;
            let bytes = unsafe { py_byte_array.as_bytes() };
            match from_utf8(bytes) {
                Ok(s) => serializer.serialize_str(s),
                Err(e) => Err(py_err_se_err(e)),
            }
        }
        ObType::Dict => {
            let py_dict: &PyDict = obj.cast_as().map_err(py_err_se_err)?;

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
            let dt_str = obj
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
            obj.cast_as::<PyDate>()
                .map_err(py_err_se_err)?
                .str()
                .map_err(py_err_se_err)?
                .to_str()
                .map_err(py_err_se_err)?,
        ),
        ObType::Time => serializer.serialize_str(
            obj.cast_as::<PyTime>()
                .map_err(py_err_se_err)?
                .str()
                .map_err(py_err_se_err)?
                .to_str()
                .map_err(py_err_se_err)?,
        ),
        _ => todo!(),
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

    fn get_type(&self, obj: &PyAny) -> ObType {
        let ob_type = obj.get_type_ptr() as usize;
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
            ObType::Unknown
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
