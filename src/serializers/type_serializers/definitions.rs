use std::borrow::Cow;

use pyo3::intern2;
use pyo3::prelude::*;
use pyo3::types::PyString;
use pyo3::types::{PyDict, PyList};

use crate::definitions::DefinitionRef;
use crate::definitions::DefinitionsBuilder;

use crate::tools::SchemaDict;

use super::{py_err_se_err, BuildSerializer, CombinedSerializer, Extra, TypeSerializer};

#[derive(Debug, Clone)]
pub struct DefinitionsSerializerBuilder;

impl BuildSerializer for DefinitionsSerializerBuilder {
    const EXPECTED_TYPE: &'static str = "definitions";

    fn build(
        schema: &Py2<'_, PyDict>,
        config: Option<&Py2<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();

        let schema_definitions: Py2<'_, PyList> = schema.get_as_req(intern2!(py, "definitions"))?;

        for schema_definition in schema_definitions {
            let schema = schema_definition.downcast()?;
            let reference = schema.get_as_req::<String>(intern2!(py, "ref"))?;
            let serializer = CombinedSerializer::build(schema, config, definitions)?;
            definitions.add_definition(reference, serializer)?;
        }

        let inner_schema = schema.get_as_req(intern2!(py, "schema"))?;
        CombinedSerializer::build(&inner_schema, config, definitions)
    }
}

#[derive(Debug, Clone)]
pub struct DefinitionRefSerializer {
    definition: DefinitionRef<CombinedSerializer>,
}

impl BuildSerializer for DefinitionRefSerializer {
    const EXPECTED_TYPE: &'static str = "definition-ref";

    fn build(
        schema: &Py2<'_, PyDict>,
        _config: Option<&Py2<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let schema_ref: Py2<'_, PyString> = schema.get_as_req(intern2!(schema.py(), "schema_ref"))?;
        let definition = definitions.get_definition(schema_ref.to_str()?);
        Ok(Self { definition }.into())
    }
}

impl_py_gc_traverse!(DefinitionRefSerializer {});

impl TypeSerializer for DefinitionRefSerializer {
    fn to_python(
        &self,
        value: &Py2<'_, PyAny>,
        include: Option<&Py2<'_, PyAny>>,
        exclude: Option<&Py2<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let comb_serializer = self.definition.get().unwrap();
        let value_id = extra.rec_guard.add(value, self.definition.id())?;
        let r = comb_serializer.to_python(value, include, exclude, extra);
        extra.rec_guard.pop(value_id, self.definition.id());
        r
    }

    fn json_key<'py>(&self, key: &Py2<'py, PyAny>, extra: &Extra) -> PyResult<Cow<'py, str>> {
        self.definition.get().unwrap().json_key(key, extra)
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Py2<'_, PyAny>,
        serializer: S,
        include: Option<&Py2<'_, PyAny>>,
        exclude: Option<&Py2<'_, PyAny>>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        let comb_serializer = self.definition.get().unwrap();
        let value_id = extra
            .rec_guard
            .add(value, self.definition.id())
            .map_err(py_err_se_err)?;
        let r = comb_serializer.serde_serialize(value, serializer, include, exclude, extra);
        extra.rec_guard.pop(value_id, self.definition.id());
        r
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }

    fn retry_with_lax_check(&self) -> bool {
        self.definition.get().unwrap().retry_with_lax_check()
    }
}
