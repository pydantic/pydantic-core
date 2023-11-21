use std::borrow::Cow;

use pyo3::intern2;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use serde::ser::Error;

use crate::build_tools::py_schema_err;
use crate::definitions::DefinitionsBuilder;
use crate::tools::SchemaDict;

use super::simple::none_json_key;
use super::string::serialize_py_str;
use super::{py_err_se_err, BuildSerializer, CombinedSerializer, Extra, PydanticSerializationError, TypeSerializer};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) enum WhenUsed {
    Always,
    UnlessNone,
    Json,
    JsonUnlessNone,
}

impl WhenUsed {
    pub fn new(schema: &Py2<'_, PyDict>, default: Self) -> PyResult<Self> {
        let when_used = schema.get_as::<Py2<'_, PyString>>(intern2!(schema.py(), "when_used"))?;
        match when_used.as_ref().map(|s| s.to_str()) {
            Some(Ok("always")) => Ok(Self::Always),
            Some(Ok("unless-none")) => Ok(Self::UnlessNone),
            Some(Ok("json")) => Ok(Self::Json),
            Some(Ok("json-unless-none")) => Ok(Self::JsonUnlessNone),
            Some(s) => py_schema_err!("Invalid value for `when_used`: {:?}", s?),
            None => Ok(default),
        }
    }

    pub fn should_use(&self, value: &Py2<'_, PyAny>, extra: &Extra) -> bool {
        match self {
            Self::Always => true,
            Self::UnlessNone => !value.is_none(),
            Self::Json => extra.mode.is_json(),
            Self::JsonUnlessNone => extra.mode.is_json() && !value.is_none(),
        }
    }

    /// Equivalent to `self.should_use` when we already know we're in JSON mode
    pub fn should_use_json(&self, value: &Py2<'_, PyAny>) -> bool {
        match self {
            Self::Always | Self::Json => true,
            _ => !value.is_none(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FormatSerializer {
    format_func: PyObject,
    formatting_string: Py<PyString>,
    when_used: WhenUsed,
}

impl BuildSerializer for FormatSerializer {
    const EXPECTED_TYPE: &'static str = "format";

    fn build(
        schema: &Py2<'_, PyDict>,
        config: Option<&Py2<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let formatting_string: Py2<'_, PyString> = schema.get_as_req(intern2!(py, "formatting_string"))?;
        let formatting_string = formatting_string.to_str()?;
        if formatting_string.is_empty() {
            ToStringSerializer::build(schema, config, definitions)
        } else {
            Ok(Self {
                format_func: py
                    .import(intern2!(py, "builtins"))?
                    .getattr(intern2!(py, "format"))?
                    .into_py(py),
                formatting_string: PyString::new2(py, formatting_string).into(),
                when_used: WhenUsed::new(schema, WhenUsed::JsonUnlessNone)?,
            }
            .into())
        }
    }
}

impl FormatSerializer {
    fn call(&self, value: &Py2<'_, PyAny>) -> Result<PyObject, String> {
        let py = value.py();
        self.format_func
            .call1(py, (value, self.formatting_string.as_ref(py)))
            .map_err(|e| {
                format!(
                    "Error calling `format(value, {})`: {}",
                    self.formatting_string
                        .attach(py)
                        .repr()
                        .unwrap_or_else(|_| intern2!(py, "???").clone()),
                    e
                )
            })
    }
}

impl_py_gc_traverse!(FormatSerializer { format_func });

impl TypeSerializer for FormatSerializer {
    fn to_python(
        &self,
        value: &Py2<'_, PyAny>,
        _include: Option<&Py2<'_, PyAny>>,
        _exclude: Option<&Py2<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        if self.when_used.should_use(value, extra) {
            self.call(value).map_err(PydanticSerializationError::new_err)
        } else {
            Ok(value.into_py(value.py()))
        }
    }

    fn json_key<'py>(&self, key: &Py2<'py, PyAny>, _extra: &Extra) -> PyResult<Cow<'py, str>> {
        if self.when_used.should_use_json(key) {
            let py_str = self
                .call(key)
                .map_err(PydanticSerializationError::new_err)?
                .attach_into(key.py())
                .downcast_into::<PyString>()?;
            Ok(Cow::Owned(py_str.to_str()?.to_owned()))
        } else {
            none_json_key()
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Py2<'_, PyAny>,
        serializer: S,
        _include: Option<&Py2<'_, PyAny>>,
        _exclude: Option<&Py2<'_, PyAny>>,
        _extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        if self.when_used.should_use_json(value) {
            match self.call(value) {
                Ok(v) => {
                    let py_str = v.attach(value.py()).downcast().map_err(py_err_se_err)?;
                    serialize_py_str(py_str, serializer)
                }
                Err(e) => Err(S::Error::custom(e)),
            }
        } else {
            serializer.serialize_none()
        }
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}

#[derive(Debug, Clone)]
pub struct ToStringSerializer {
    when_used: WhenUsed,
}

impl BuildSerializer for ToStringSerializer {
    const EXPECTED_TYPE: &'static str = "to-string";

    fn build(
        schema: &Py2<'_, PyDict>,
        _config: Option<&Py2<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        Ok(Self {
            when_used: WhenUsed::new(schema, WhenUsed::JsonUnlessNone)?,
        }
        .into())
    }
}

impl_py_gc_traverse!(ToStringSerializer {});

impl TypeSerializer for ToStringSerializer {
    fn to_python(
        &self,
        value: &Py2<'_, PyAny>,
        _include: Option<&Py2<'_, PyAny>>,
        _exclude: Option<&Py2<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        if self.when_used.should_use(value, extra) {
            value.str().map(|s| s.into_py(value.py()))
        } else {
            Ok(value.into_py(value.py()))
        }
    }

    fn json_key<'py>(&self, key: &Py2<'py, PyAny>, _extra: &Extra) -> PyResult<Cow<'py, str>> {
        if self.when_used.should_use_json(key) {
            Ok(Cow::Owned(key.str()?.to_string_lossy().into_owned()))
        } else {
            none_json_key()
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Py2<'_, PyAny>,
        serializer: S,
        _include: Option<&Py2<'_, PyAny>>,
        _exclude: Option<&Py2<'_, PyAny>>,
        _extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        if self.when_used.should_use_json(value) {
            let s = value.str().map_err(py_err_se_err)?;
            serialize_py_str(&s, serializer)
        } else {
            serializer.serialize_none()
        }
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}
