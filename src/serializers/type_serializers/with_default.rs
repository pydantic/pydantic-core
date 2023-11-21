use std::borrow::Cow;

use pyo3::intern2;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::definitions::DefinitionsBuilder;
use crate::tools::SchemaDict;
use crate::validators::DefaultType;

use super::{BuildSerializer, CombinedSerializer, Extra, TypeSerializer};

#[derive(Debug, Clone)]
pub struct WithDefaultSerializer {
    default: DefaultType,
    serializer: Box<CombinedSerializer>,
}

impl BuildSerializer for WithDefaultSerializer {
    const EXPECTED_TYPE: &'static str = "default";

    fn build(
        schema: &Py2<'_, PyDict>,
        config: Option<&Py2<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let default = DefaultType::new(schema)?;

        let sub_schema = schema.get_as_req(intern2!(py, "schema"))?;
        let serializer = Box::new(CombinedSerializer::build(&sub_schema, config, definitions)?);

        Ok(Self { default, serializer }.into())
    }
}

impl_py_gc_traverse!(WithDefaultSerializer { default, serializer });

impl TypeSerializer for WithDefaultSerializer {
    fn to_python(
        &self,
        value: &Py2<'_, PyAny>,
        include: Option<&Py2<'_, PyAny>>,
        exclude: Option<&Py2<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        self.serializer.to_python(value, include, exclude, extra)
    }

    fn json_key<'py>(&self, key: &Py2<'py, PyAny>, extra: &Extra) -> PyResult<Cow<'py, str>> {
        self.serializer.json_key(key, extra)
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Py2<'_, PyAny>,
        serializer: S,
        include: Option<&Py2<'_, PyAny>>,
        exclude: Option<&Py2<'_, PyAny>>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        self.serializer
            .serde_serialize(value, serializer, include, exclude, extra)
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }

    fn retry_with_lax_check(&self) -> bool {
        self.serializer.retry_with_lax_check()
    }

    fn get_default(&self, py: Python) -> PyResult<Option<PyObject>> {
        self.default.default_value(py)
    }
}
