use std::borrow::Cow;
use std::str::from_utf8;

use pyo3::prelude::*;
use pyo3::types::{
    PyByteArray, PyBytes, PyDate, PyDateTime, PyDelta, PyDict, PyFrozenSet, PyList, PySet, PyString, PyTime, PyTuple,
};

use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};

use crate::build_context::BuildContext;
use crate::url::{PyMultiHostUrl, PyUrl};

use super::{
    py_err_se_err, utf8_py_error, BuildSerializer, CombinedSerializer, Extra, ObType, SerMode, TypeSerializer,
};

#[derive(Debug, Clone)]
pub struct AnySerializer;

impl BuildSerializer for AnySerializer {
    const EXPECTED_TYPE: &'static str = "any";

    fn build(
        _schema: &PyDict,
        _config: Option<&PyDict>,
        _build_context: &mut BuildContext<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
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
        SerializeInfer::new(value, extra).serialize(serializer)
    }
}

pub(crate) fn fallback_to_python(value: &PyAny, extra: &Extra) -> PyResult<PyObject> {
    match extra.mode {
        SerMode::Json => fallback_to_python_json(value, extra),
        _ => Ok(value.into_py(value.py())),
    }
}

pub(crate) fn fallback_to_python_json(value: &PyAny, extra: &Extra) -> PyResult<PyObject> {
    ob_type_to_python_json(&extra.ob_type_lookup.get_type(value), value, extra)
}

pub(crate) fn ob_type_to_python_json(ob_type: &ObType, value: &PyAny, extra: &Extra) -> PyResult<PyObject> {
    let value_id = extra.rec_guard.add(value)?;
    let py = value.py();

    // have to do this to make sure subclasses of for example str are upcast to `str`
    macro_rules! extract_as {
        ($t:ty) => {{
            let v: $t = value.extract()?;
            v.into_py(py)
        }};
    }

    macro_rules! serialize_seq {
        ($t:ty) => {{
            let vec: Vec<PyObject> = value
                .cast_as::<$t>()?
                .iter()
                .map(|v| fallback_to_python_json(v, extra))
                .collect::<PyResult<_>>()?;
            PyList::new(py, vec).into_py(py)
        }};
    }

    let value = match ob_type {
        ObType::Int => extract_as!(i64),
        // `bool` and `None` can't be subclasses, so no need to do the same on bool
        ObType::Float => extract_as!(f64),
        ObType::Str => extract_as!(&str),
        ObType::Bytes => extra
            .config
            .bytes_mode
            .bytes_to_string(value.cast_as()?)
            .map(|s| s.into_py(py))?,
        ObType::Bytearray => {
            let py_byte_array: &PyByteArray = value.cast_as()?;
            // see https://docs.rs/pyo3/latest/pyo3/types/struct.PyByteArray.html#method.as_bytes
            // for why this is marked unsafe
            let bytes = unsafe { py_byte_array.as_bytes() };
            match from_utf8(bytes) {
                Ok(s) => s.into_py(py),
                Err(err) => return Err(utf8_py_error(py, err, bytes)),
            }
        }
        ObType::Tuple => serialize_seq!(PyTuple),
        ObType::List => serialize_seq!(PyList),
        ObType::Set => serialize_seq!(PySet),
        ObType::Frozenset => serialize_seq!(PyFrozenSet),
        ObType::Dict => {
            let dict: &PyDict = value.cast_as()?;
            let new_dict = PyDict::new(py);
            for (k, v) in dict {
                let k_str = json_key(k, extra)?;
                let k = PyString::new(py, &k_str);
                let v = fallback_to_python_json(v, extra)?;
                new_dict.set_item(k, v)?;
            }
            new_dict.into_py(py)
        }
        ObType::Datetime => {
            let py_dt: &PyDateTime = value.cast_as()?;
            let iso_dt = super::datetime_etc::datetime_to_string(py_dt)?;
            iso_dt.into_py(py)
        }
        ObType::Date => {
            let py_date: &PyDate = value.cast_as()?;
            let iso_date = super::datetime_etc::date_to_string(py_date)?;
            iso_date.into_py(py)
        }
        ObType::Time => {
            let py_time: &PyTime = value.cast_as()?;
            let iso_time = super::datetime_etc::time_to_string(py_time)?;
            iso_time.into_py(py)
        }
        ObType::Timedelta => {
            let py_timedelta: &PyDelta = value.cast_as()?;
            extra.config.timedelta_mode.timedelta_to_json(py_timedelta)?
        }
        ObType::Url => {
            let py_url: PyUrl = value.extract()?;
            py_url.__str__().into_py(py)
        }
        ObType::MultiHostUrl => {
            let py_url: PyMultiHostUrl = value.extract()?;
            py_url.__str__().into_py(py)
        }
        // TODO error here we
        _ => value.into_py(value.py()),
    };
    extra.rec_guard.pop(value_id);
    Ok(value)
}

pub(crate) struct SerializeInfer<'py> {
    value: &'py PyAny,
    extra: &'py Extra<'py>,
}

impl<'py> SerializeInfer<'py> {
    pub(crate) fn new(value: &'py PyAny, extra: &'py Extra) -> Self {
        Self { value, extra }
    }
}

impl<'py> Serialize for SerializeInfer<'py> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let ob_type = self.extra.ob_type_lookup.get_type(self.value);
        fallback_serialize_known(&ob_type, self.value, serializer, self.extra)
    }
}

pub(crate) fn fallback_serialize<S: Serializer>(
    value: &PyAny,
    serializer: S,
    extra: &Extra,
) -> Result<S::Ok, S::Error> {
    fallback_serialize_known(&extra.ob_type_lookup.get_type(value), value, serializer, extra)
}

pub(crate) fn fallback_serialize_known<S: Serializer>(
    ob_type: &ObType,
    value: &PyAny,
    serializer: S,
    extra: &Extra,
) -> Result<S::Ok, S::Error> {
    let value_id = extra.rec_guard.add(value).map_err(py_err_se_err)?;
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
            let py_seq: &$t = value.cast_as().map_err(py_err_se_err)?;
            let mut seq = serializer.serialize_seq(Some(py_seq.len()))?;
            for element in py_seq {
                seq.serialize_element(&SerializeInfer::new(element, extra))?
            }
            seq.end()
        }};
    }

    let ser_result = match ob_type {
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
            extra.config.bytes_mode.serialize_bytes(py_bytes, serializer)
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
            for (key, value) in py_dict {
                let key = json_key(key, extra).map_err(py_err_se_err)?;
                map.serialize_entry(&key, &SerializeInfer::new(value, extra))?;
            }
            map.end()
        }
        ObType::List => serialize_seq!(PyList),
        ObType::Tuple => serialize_seq!(PyTuple),
        ObType::Set => serialize_seq!(PySet),
        ObType::Frozenset => serialize_seq!(PyFrozenSet),
        ObType::Datetime => {
            let py_dt: &PyDateTime = value.cast_as().map_err(py_err_se_err)?;
            let iso_dt = super::datetime_etc::datetime_to_string(py_dt).map_err(py_err_se_err)?;
            serializer.serialize_str(&iso_dt)
        }
        ObType::Date => {
            let py_date: &PyDate = value.cast_as().map_err(py_err_se_err)?;
            let iso_date = super::datetime_etc::date_to_string(py_date).map_err(py_err_se_err)?;
            serializer.serialize_str(&iso_date)
        }
        ObType::Time => {
            let py_time: &PyTime = value.cast_as().map_err(py_err_se_err)?;
            let iso_time = super::datetime_etc::time_to_string(py_time).map_err(py_err_se_err)?;
            serializer.serialize_str(&iso_time)
        }
        ObType::Timedelta => {
            let py_timedelta: &PyDelta = value.cast_as().map_err(py_err_se_err)?;
            extra
                .config
                .timedelta_mode
                .timedelta_serialize(py_timedelta, serializer)
        }
        ObType::Url => {
            let py_url: PyUrl = value.extract().map_err(py_err_se_err)?;
            serializer.serialize_str(py_url.__str__())
        }
        ObType::MultiHostUrl => {
            let py_url: PyMultiHostUrl = value.extract().map_err(py_err_se_err)?;
            serializer.serialize_str(&py_url.__str__())
        }
        _ => todo!(),
        // _ => serializer.serialize_none(),
    };
    extra.rec_guard.pop(value_id);
    ser_result
}

pub(crate) fn json_key<'py>(key: &'py PyAny, extra: &Extra) -> PyResult<Cow<'py, str>> {
    let ob_type = extra.ob_type_lookup.get_type(key);

    match ob_type {
        ObType::None => Ok(Cow::Borrowed("None")),
        ObType::Bool => {
            let v = if key.is_true().unwrap_or(false) {
                "true"
            } else {
                "false"
            };
            Ok(Cow::Borrowed(v))
        }
        ObType::Str => {
            let py_str: &PyString = key.cast_as()?;
            Ok(Cow::Borrowed(py_str.to_str()?))
        }
        ObType::Bytes => extra.config.bytes_mode.bytes_to_string(key.cast_as()?),
        // perhaps we could do something faster for things like ints and floats?
        ObType::Datetime => {
            let py_dt: &PyDateTime = key.cast_as()?;
            let iso_dt = super::datetime_etc::datetime_to_string(py_dt)?;
            Ok(Cow::Owned(iso_dt))
        }
        ObType::Date => {
            let py_date: &PyDate = key.cast_as()?;
            let iso_date = super::datetime_etc::date_to_string(py_date)?;
            Ok(Cow::Owned(iso_date))
        }
        ObType::Time => {
            let py_time: &PyTime = key.cast_as()?;
            let iso_time = super::datetime_etc::time_to_string(py_time)?;
            Ok(Cow::Owned(iso_time))
        }
        ObType::Timedelta => {
            let py_timedelta: &PyDelta = key.cast_as()?;
            extra.config.timedelta_mode.json_key(py_timedelta)
        }
        ObType::Url => {
            let py_url: PyUrl = key.extract()?;
            Ok(Cow::Owned(py_url.__str__().to_string()))
        }
        ObType::MultiHostUrl => {
            let py_url: PyMultiHostUrl = key.extract()?;
            Ok(Cow::Owned(py_url.__str__()))
        }
        _ => Ok(key.str()?.to_string_lossy()),
    }
}
