use std::borrow::Cow;
use std::str::{from_utf8, Utf8Error};

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

use serde::ser::Error;

use crate::build_tools::SchemaDict;

use super::any::{fallback_serialize, fallback_to_python_json, json_key};
use super::shared::{BuildSerializer, CombinedSerializer, Extra, SerMode, TypeSerializer};

#[derive(Debug, Clone)]
pub struct BytesSerializer {
    base64_config: Option<base64::Config>,
}

impl BuildSerializer for BytesSerializer {
    const EXPECTED_TYPE: &'static str = "bytes";

    fn build(schema: &PyDict, _config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let json_base64: bool = match schema.get_as::<&PyDict>(intern!(py, "serialization"))? {
            Some(ser) => ser.get_as(intern!(py, "json_base64"))?.unwrap_or(false),
            None => false,
        };
        let base64_config = if json_base64 {
            Some(base64::Config::new(base64::CharacterSet::UrlSafe, true))
        } else {
            None
        };
        Ok(Self { base64_config }.into())
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
                Ok(py_bytes) => {
                    if let Some(config) = self.base64_config {
                        Ok(base64::encode_config(py_bytes.as_bytes(), config).into_py(py))
                    } else {
                        py_bytes_to_str(py_bytes).map(|s| s.into_py(py))
                    }
                }
                Err(_) => {
                    extra.warnings.fallback_slow(Self::EXPECTED_TYPE, value);
                    fallback_to_python_json(value, extra)
                }
            },
            _ => Ok(value.into_py(py)),
        }
    }

    fn json_key<'py>(&self, key: &'py PyAny, extra: &Extra) -> PyResult<Cow<'py, str>> {
        match key.cast_as::<PyBytes>() {
            Ok(py_bytes) => {
                if let Some(config) = self.base64_config {
                    Ok(Cow::Owned(base64::encode_config(py_bytes.as_bytes(), config)))
                } else {
                    py_bytes_to_str(py_bytes).map(Cow::Borrowed)
                }
            }
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
            Ok(py_bytes) => {
                if let Some(config) = self.base64_config {
                    serializer.serialize_str(&base64::encode_config(py_bytes.as_bytes(), config))
                } else {
                    serialize_py_bytes(py_bytes, serializer)
                }
            }
            Err(_) => {
                extra.warnings.fallback_slow(Self::EXPECTED_TYPE, value);
                fallback_serialize(value, serializer, extra)
            }
        }
    }
}

pub fn utf8_py_error(py: Python, err: Utf8Error, data: &[u8]) -> PyErr {
    #[cfg(not(PyPy))]
    return match pyo3::exceptions::PyUnicodeDecodeError::new_utf8(py, data, err) {
        Ok(decode_err) => PyErr::from_value(decode_err),
        Err(err) => err,
    };
    // See https://github.com/PyO3/pyo3/issues/2770
    #[cfg(PyPy)]
    pyo3::exceptions::PyValueError::new_err(err.to_string())
}

pub fn py_bytes_to_str(py_bytes: &PyBytes) -> PyResult<&str> {
    let py = py_bytes.py();
    let data = py_bytes.as_bytes();
    from_utf8(data).map_err(|err| utf8_py_error(py, err, data))
}

pub fn serialize_py_bytes<S: serde::ser::Serializer>(py_bytes: &PyBytes, serializer: S) -> Result<S::Ok, S::Error> {
    match from_utf8(py_bytes.as_bytes()) {
        Ok(s) => serializer.serialize_str(s),
        Err(e) => Err(Error::custom(e.to_string())),
    }
}
