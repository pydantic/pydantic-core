use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use super::any::{fallback_serialize, fallback_to_python_json, IsType, ObType};
use super::shared::{py_err_se_err, BuildSerializer, CombinedSerializer, Extra, SerFormat, TypeSerializer};

#[derive(Debug, Clone)]
pub struct StrSerializer;

impl BuildSerializer for StrSerializer {
    const EXPECTED_TYPE: &'static str = "str";

    fn build(_schema: &PyDict, _config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        Ok(Self {}.into())
    }
}

impl TypeSerializer for StrSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let py = value.py();
        match extra.format {
            SerFormat::Json => match extra.ob_type_lookup.is_type(value, ObType::Str) {
                IsType::Exact => Ok(value.into_py(py)),
                IsType::Subclass => {
                    let s: &str = value.extract()?;
                    Ok(s.into_py(py))
                }
                IsType::False => {
                    extra.warnings.fallback_slow(Self::EXPECTED_TYPE, value);
                    fallback_to_python_json(value, extra.ob_type_lookup)
                }
            },
            _ => Ok(value.into_py(py)),
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
        match value.cast_as::<PyString>() {
            Ok(py_str) => serialize_py_str(py_str, serializer),
            Err(_) => {
                extra.warnings.fallback_slow(Self::EXPECTED_TYPE, value);
                fallback_serialize(value, serializer, extra.ob_type_lookup)
            }
        }
    }
}

pub fn serialize_py_str<S: serde::ser::Serializer>(py_str: &PyString, serializer: S) -> Result<S::Ok, S::Error> {
    let s = py_str.to_str().map_err(py_err_se_err)?;
    serializer.serialize_str(s)
}
