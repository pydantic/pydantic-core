use std::borrow::Cow;
use std::str::from_utf8;

use pyo3::exceptions::PyUnicodeDecodeError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

use serde::ser::Error;

use super::any::{fallback_serialize, fallback_to_python_json, json_key};
use super::shared::{BuildSerializer, CombinedSerializer, Extra, SerMode, TypeSerializer};

#[derive(Debug, Clone)]
pub struct BytesSerializer;

impl BuildSerializer for BytesSerializer {
    const EXPECTED_TYPE: &'static str = "bytes";

    fn build(_schema: &PyDict, _config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        Ok(Self {}.into())
    }
}

impl TypeSerializer for BytesSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let py = value.py();
        match extra.mode {
            SerMode::Json => match value.cast_as::<PyBytes>() {
                Ok(py_bytes) => py_bytes_to_str(py_bytes).map(|s| s.into_py(py)),
                Err(_) => {
                    extra.warnings.fallback_slow(Self::EXPECTED_TYPE, value);
                    fallback_to_python_json(value, extra.ob_type_lookup)
                }
            },
            _ => Ok(value.into_py(py)),
        }
    }

    fn json_key<'py>(&self, key: &'py PyAny, extra: &Extra) -> PyResult<Cow<'py, str>> {
        match key.cast_as::<PyBytes>() {
            Ok(py_bytes) => py_bytes_to_str(py_bytes).map(Cow::Borrowed),
            Err(_) => {
                extra.warnings.fallback_slow(Self::EXPECTED_TYPE, key);
                json_key(key, extra)
            }
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        match value.cast_as::<PyBytes>() {
            Ok(py_bytes) => serialize_py_bytes(py_bytes, serializer),
            Err(_) => {
                extra.warnings.fallback_slow(Self::EXPECTED_TYPE, value);
                fallback_serialize(value, serializer, extra.ob_type_lookup)
            }
        }
    }
}

pub fn py_bytes_to_str(py_bytes: &PyBytes) -> PyResult<&str> {
    match from_utf8(py_bytes.as_bytes()) {
        Ok(s) => Ok(s),
        Err(err) => {
            let py = py_bytes.py();
            let decode_err = PyUnicodeDecodeError::new_utf8(py, py_bytes.as_bytes(), err)?;
            Err(PyErr::from_value(decode_err))
        }
    }
}

pub fn serialize_py_bytes<S: serde::ser::Serializer>(py_bytes: &PyBytes, serializer: S) -> Result<S::Ok, S::Error> {
    match from_utf8(py_bytes.as_bytes()) {
        Ok(s) => serializer.serialize_str(s),
        Err(e) => Err(Error::custom(e.to_string())),
    }
}
