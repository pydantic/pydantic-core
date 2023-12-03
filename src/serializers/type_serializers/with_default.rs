use std::borrow::Cow;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::definitions::DefinitionsBuilder;
use crate::tools::SchemaDict;
use crate::validators::DefaultType;

use super::{BuildSerializer, CombinedSerializer, Extra, TypeSerializer};

#[derive(Debug, Clone)]
pub struct WithDefaultSerializer {
    default: DefaultType,
    default_comparison: Option<PyObject>,
    serializer: Box<CombinedSerializer>,
}

impl BuildSerializer for WithDefaultSerializer {
    const EXPECTED_TYPE: &'static str = "default";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let default = DefaultType::new(schema)?;
        let default_comparison = schema.get_as(intern!(py, "default_comparison"))?;
        let sub_schema: &PyDict = schema.get_as_req(intern!(py, "schema"))?;
        let serializer = Box::new(CombinedSerializer::build(sub_schema, config, definitions)?);

        Ok(Self {
            default,
            default_comparison,
            serializer,
        }
        .into())
    }
}

impl_py_gc_traverse!(WithDefaultSerializer { default, serializer });

impl TypeSerializer for WithDefaultSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        self.serializer.to_python(value, include, exclude, extra)
    }

    fn json_key<'py>(&self, key: &'py PyAny, extra: &Extra) -> PyResult<Cow<'py, str>> {
        self.serializer.json_key(key, extra)
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
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

    fn compare_with_default(&self, py: Python, value: &PyAny) -> PyResult<bool> {
        if let Some(default) = self.get_default(py)? {
            if let Some(default_comparison) = &self.default_comparison {
                return default_comparison.call(py, (value, default), None)?.extract::<bool>(py);
            } else if value.eq(default)? {
                return Ok(true);
            }
        }

        Ok(false)
    }
}
