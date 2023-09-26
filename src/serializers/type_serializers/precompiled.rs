use std::borrow::Cow;

use pyo3::types::PyDict;
use pyo3::{intern, prelude::*};

use crate::build_tools::py_schema_err;
use crate::definitions::DefinitionsBuilder;
use crate::serializers::shared::TypeSerializer;
use crate::serializers::Extra;
use crate::tools::SchemaDict;
use crate::SchemaSerializer;

use super::{BuildSerializer, CombinedSerializer};

#[derive(Debug, Clone)]
pub struct PrecompiledSerializer {
    serializer: Py<SchemaSerializer>,
}

impl BuildSerializer for PrecompiledSerializer {
    const EXPECTED_TYPE: &'static str = "precompiled";

    fn build(
        schema: &PyDict,
        _config: Option<&PyDict>,
        _definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let sub_schema: &PyAny = schema.get_as_req(intern!(py, "schema"))?;
        let serializer: PyRef<SchemaSerializer> = schema.get_as_req(intern!(py, "serializer"))?;

        // TODO DEBUG THIS LATER
        // if !serializer.schema.is(sub_schema) {
        //     return py_schema_err!("precompiled schema mismatch");
        // }

        Ok(CombinedSerializer::Precompiled(PrecompiledSerializer {
            serializer: serializer.into(),
        }))
    }
}

impl_py_gc_traverse!(PrecompiledSerializer { serializer });

impl TypeSerializer for PrecompiledSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        self.serializer
            .get()
            .serializer
            .to_python(value, include, exclude, extra)
    }

    fn json_key<'py>(&self, key: &'py PyAny, extra: &Extra) -> PyResult<Cow<'py, str>> {
        self.serializer.get().serializer.json_key(key, extra)
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
            .get()
            .serializer
            .serde_serialize(value, serializer, include, exclude, extra)
    }

    fn get_name(&self) -> &str {
        self.serializer.get().serializer.get_name()
    }
}
