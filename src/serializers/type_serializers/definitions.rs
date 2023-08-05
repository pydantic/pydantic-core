use std::borrow::Cow;
use std::fmt::Debug;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::definitions::{DefinitionRef, DefinitionsBuilder};

use crate::tools::SchemaDict;

use super::{py_err_se_err, BuildSerializer, CombinedSerializer, Extra, TypeSerializer};

#[derive(Debug, Clone)]
pub struct DefinitionsSerializerBuilder;

impl BuildSerializer for DefinitionsSerializerBuilder {
    const EXPECTED_TYPE: &'static str = "definitions";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();

        let schema_definitions: &PyList = schema.get_as_req(intern!(py, "definitions"))?;

        for schema_def in schema_definitions {
            CombinedSerializer::build(schema_def.downcast()?, config, definitions)?;
            // no need to store the serializer here, it has already been stored in definitions if necessary
        }

        let inner_schema: &PyDict = schema.get_as_req(intern!(py, "schema"))?;
        CombinedSerializer::build(inner_schema, config, definitions)
    }
}

#[derive(Debug, Clone)]
pub struct DefinitionRefSerializer {
    definition: DefinitionRef<CombinedSerializer>,
}

impl DefinitionRefSerializer {
    pub fn new(definition: DefinitionRef<CombinedSerializer>) -> DefinitionRefSerializer {
        Self { definition }
    }
}

impl BuildSerializer for DefinitionRefSerializer {
    const EXPECTED_TYPE: &'static str = "definition-ref";

    fn build(
        schema: &PyDict,
        _config: Option<&PyDict>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let schema_ref: String = schema.get_as_req(intern!(schema.py(), "schema_ref"))?;
        let definition = definitions.get_definition(&schema_ref).clone();
        Ok(Self { definition }.into())
    }
}

// NB it is NOT correct to traverse the definition here; doing so may lead to circular references.
// Instead leave the traversal to the definitions in the top-level schema validator.
impl_py_gc_traverse!(DefinitionRefSerializer {});

impl TypeSerializer for DefinitionRefSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let value_id = extra.rec_guard.add(value, self.definition.id())?;
        let comb_serializer = self.definition.get().unwrap();
        let r = comb_serializer.to_python(value, include, exclude, extra);
        extra.rec_guard.pop(value_id, self.definition.id());
        r
    }

    fn json_key<'py>(&self, key: &'py PyAny, extra: &Extra) -> PyResult<Cow<'py, str>> {
        self._invalid_as_json_key(key, extra, Self::EXPECTED_TYPE)
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        let value_id = extra
            .rec_guard
            .add(value, self.definition.id())
            .map_err(py_err_se_err)?;
        let comb_serializer = self.definition.get().unwrap();
        let r = comb_serializer.serde_serialize(value, serializer, include, exclude, extra);
        extra.rec_guard.pop(value_id, self.definition.id());
        r
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}
