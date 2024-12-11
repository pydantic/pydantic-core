use super::{py_err_se_err, BuildSerializer, CombinedSerializer, Extra, TypeSerializer};
use crate::definitions::DefinitionsBuilder;
use crate::errors::ErrorTypeDefaults;
use crate::tools::py_err;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::borrow::Cow;

#[derive(Debug)]
pub struct NeverSerializer;

impl BuildSerializer for NeverSerializer {
    const EXPECTED_TYPE: &'static str = "never";

    fn build(
        _schema: &Bound<'_, PyDict>,
        _config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        Ok(Self {}.into())
    }
}

impl_py_gc_traverse!(NeverSerializer {});

impl TypeSerializer for NeverSerializer {
    fn to_python(
        &self,
        _value: &Bound<'_, PyAny>,
        _include: Option<&Bound<'_, PyAny>>,
        _exclude: Option<&Bound<'_, PyAny>>,
        _extra: &Extra,
    ) -> PyResult<PyObject> {
        py_err!(PyTypeError; ErrorTypeDefaults::Never.message_template_python())
    }

    fn json_key<'a>(&self, _key: &'a Bound<'_, PyAny>, _extra: &Extra) -> PyResult<Cow<'a, str>> {
        py_err!(PyTypeError; ErrorTypeDefaults::Never.message_template_python())
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        _value: &Bound<'_, PyAny>,
        _serializer: S,
        _include: Option<&Bound<'_, PyAny>>,
        _exclude: Option<&Bound<'_, PyAny>>,
        _extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        py_err!(PyTypeError; ErrorTypeDefaults::Never.message_template_python()).map_err(py_err_se_err)
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}
