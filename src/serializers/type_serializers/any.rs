use std::borrow::Cow;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use serde::ser::Serializer;

use crate::definitions::DefinitionsBuilder;

use super::{
    infer_json_key, infer_serialize, infer_to_python, BuildSerializer, CombinedSerializer, Extra, TypeSerializer,
};

#[derive(Debug, Clone, Default)]
pub struct AnySerializer;

impl BuildSerializer for AnySerializer {
    const EXPECTED_TYPE: &'static str = "any";

    fn build(
        _schema: &Py2<'_, PyDict>,
        _config: Option<&Py2<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        Ok(Self {}.into())
    }
}

impl_py_gc_traverse!(AnySerializer {});

impl TypeSerializer for AnySerializer {
    fn to_python(
        &self,
        value: &Py2<'_, PyAny>,
        include: Option<&Py2<'_, PyAny>>,
        exclude: Option<&Py2<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        infer_to_python(value, include, exclude, extra)
    }

    fn json_key<'py>(&self, key: &Py2<'py, PyAny>, extra: &Extra) -> PyResult<Cow<'py, str>> {
        infer_json_key(key, extra)
    }

    fn serde_serialize<S: Serializer>(
        &self,
        value: &Py2<'_, PyAny>,
        serializer: S,
        include: Option<&Py2<'_, PyAny>>,
        exclude: Option<&Py2<'_, PyAny>>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        infer_serialize(value, serializer, include, exclude, extra)
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}
