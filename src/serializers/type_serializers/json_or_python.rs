use std::borrow::Cow;

use pyo3::intern2;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use super::{BuildSerializer, CombinedSerializer, Extra, TypeSerializer};
use crate::definitions::DefinitionsBuilder;
use crate::tools::SchemaDict;

#[derive(Debug, Clone)]
pub struct JsonOrPythonSerializer {
    json: Box<CombinedSerializer>,
    python: Box<CombinedSerializer>,
    name: String,
}

impl BuildSerializer for JsonOrPythonSerializer {
    const EXPECTED_TYPE: &'static str = "json-or-python";

    fn build(
        schema: &Py2<'_, PyDict>,
        config: Option<&Py2<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let json_schema = schema.get_as_req(intern2!(py, "json_schema"))?;
        let python_schema = schema.get_as_req(intern2!(py, "python_schema"))?;

        let json = CombinedSerializer::build(&json_schema, config, definitions)?;
        let python = CombinedSerializer::build(&python_schema, config, definitions)?;

        let name = format!(
            "{}[json={}, python={}]",
            Self::EXPECTED_TYPE,
            json.get_name(),
            python.get_name(),
        );
        Ok(Self {
            json: Box::new(json),
            python: Box::new(python),
            name,
        }
        .into())
    }
}

impl_py_gc_traverse!(JsonOrPythonSerializer { json, python });

impl TypeSerializer for JsonOrPythonSerializer {
    fn to_python(
        &self,
        value: &Py2<'_, PyAny>,
        include: Option<&Py2<'_, PyAny>>,
        exclude: Option<&Py2<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        self.python.to_python(value, include, exclude, extra)
    }

    fn json_key<'py>(&self, key: &Py2<'py, PyAny>, extra: &Extra) -> PyResult<Cow<'py, str>> {
        self.json.json_key(key, extra)
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Py2<'_, PyAny>,
        serializer: S,
        include: Option<&Py2<'_, PyAny>>,
        exclude: Option<&Py2<'_, PyAny>>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        self.json.serde_serialize(value, serializer, include, exclude, extra)
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}
