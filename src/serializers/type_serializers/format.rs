use std::borrow::Cow;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use serde::ser::Error;

use crate::build_context::BuildContext;
use crate::build_tools::SchemaDict;

use super::simple::none_json_key;
use super::string::serialize_py_str;
use super::{py_err_se_err, BuildSerializer, CombinedSerializer, Extra, PydanticSerializationError, TypeSerializer};

#[derive(Debug, Clone)]
pub struct FormatSerializer {
    format_func: PyObject,
    formatting_string: Py<PyString>,
    format_to_python: bool,
}

impl BuildSerializer for FormatSerializer {
    const EXPECTED_TYPE: &'static str = "format";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let formatting_string: &str = schema.get_as_req(intern!(py, "formatting_string"))?;
        if formatting_string.is_empty() {
            ToStringSerializer::build(schema, config, build_context)
        } else {
            Ok(Self {
                format_func: py
                    .import(intern!(py, "builtins"))?
                    .getattr(intern!(py, "format"))?
                    .into_py(py),
                formatting_string: PyString::new(py, formatting_string).into_py(py),
                format_to_python: schema.get_as(intern!(py, "format_to_python"))?.unwrap_or(false),
            }
            .into())
        }
    }
}
impl FormatSerializer {
    fn call(&self, value: &PyAny) -> Result<PyObject, String> {
        let py = value.py();
        self.format_func
            .call1(py, (value, self.formatting_string.as_ref(py)))
            .map_err(|e| {
                format!(
                    "Error calling `format(value, {})`: {}",
                    self.formatting_string
                        .as_ref(py)
                        .repr()
                        .unwrap_or_else(|_| intern!(py, "???")),
                    e
                )
            })
    }
}

impl TypeSerializer for FormatSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        if extra.mode.is_json() || self.format_to_python {
            if value.is_none() {
                Ok(value.into_py(value.py()))
            } else {
                self.call(value).map_err(PydanticSerializationError::new_err)
            }
        } else {
            Ok(value.into_py(value.py()))
        }
    }

    fn json_key<'py>(&self, key: &'py PyAny, _extra: &Extra) -> PyResult<Cow<'py, str>> {
        if key.is_none() {
            none_json_key()
        } else {
            let v = self.call(key).map_err(PydanticSerializationError::new_err)?;
            let py_str: &PyString = v.into_ref(key.py()).downcast()?;
            Ok(Cow::Borrowed(py_str.to_str()?))
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
        _extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        if value.is_none() {
            serializer.serialize_none()
        } else {
            match self.call(value) {
                Ok(v) => {
                    let py_str: &PyString = v.downcast(value.py()).map_err(py_err_se_err)?;
                    serialize_py_str(py_str, serializer)
                }
                Err(e) => Err(S::Error::custom(e)),
            }
        }
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}

#[derive(Debug, Clone)]
pub struct ToStringSerializer {
    format_to_python: bool,
}

impl BuildSerializer for ToStringSerializer {
    const EXPECTED_TYPE: &'static str = "to-string";

    fn build(
        schema: &PyDict,
        _config: Option<&PyDict>,
        _build_context: &mut BuildContext<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        Ok(Self {
            format_to_python: schema
                .get_as(intern!(schema.py(), "format_to_python"))?
                .unwrap_or(false),
        }
        .into())
    }
}

impl TypeSerializer for ToStringSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        if extra.mode.is_json() || self.format_to_python {
            if value.is_none() {
                Ok(value.into_py(value.py()))
            } else {
                value.str().map(|s| s.into_py(value.py()))
            }
        } else {
            Ok(value.into_py(value.py()))
        }
    }

    fn json_key<'py>(&self, key: &'py PyAny, _extra: &Extra) -> PyResult<Cow<'py, str>> {
        if key.is_none() {
            none_json_key()
        } else {
            Ok(key.str()?.to_string_lossy())
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
        _extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        if value.is_none() {
            serializer.serialize_none()
        } else {
            let s = value.str().map_err(py_err_se_err)?;
            serialize_py_str(s, serializer)
        }
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}
