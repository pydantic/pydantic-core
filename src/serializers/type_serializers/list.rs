use std::borrow::Cow;
use std::sync::Arc;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use pyo3::IntoPyObjectExt;
use serde::ser::SerializeSeq;

use crate::definitions::DefinitionsBuilder;
use crate::serializers::SerializationState;
use crate::tools::SchemaDict;

use super::any::AnySerializer;
use super::{
    infer_serialize, infer_to_python, py_err_se_err, BuildSerializer, CombinedSerializer, Extra, PydanticSerializer,
    SchemaFilter, TypeSerializer,
};

#[derive(Debug)]
pub struct ListSerializer {
    item_serializer: Arc<CombinedSerializer>,
    filter: SchemaFilter<usize>,
    name: String,
}

impl BuildSerializer for ListSerializer {
    const EXPECTED_TYPE: &'static str = "list";

    fn build(
        schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<Arc<CombinedSerializer>>,
    ) -> PyResult<Arc<CombinedSerializer>> {
        let py = schema.py();
        let item_serializer = match schema.get_as(intern!(py, "items_schema"))? {
            Some(items_schema) => CombinedSerializer::build(&items_schema, config, definitions)?,
            None => AnySerializer::build(schema, config, definitions)?,
        };
        let name = format!("{}[{}]", Self::EXPECTED_TYPE, item_serializer.get_name());
        Ok(Arc::new(
            Self {
                item_serializer,
                filter: SchemaFilter::from_schema(schema)?,
                name,
            }
            .into(),
        ))
    }
}

impl_py_gc_traverse!(ListSerializer { item_serializer });

impl TypeSerializer for ListSerializer {
    fn to_python<'py>(
        &self,
        value: &Bound<'py, PyAny>,
        state: &mut SerializationState<'py>,
        extra: &Extra<'_, 'py>,
    ) -> PyResult<Py<PyAny>> {
        match value.downcast::<PyList>() {
            Ok(py_list) => {
                let py = value.py();
                let item_serializer = self.item_serializer.as_ref();

                let mut items = Vec::with_capacity(py_list.len());
                for (index, element) in py_list.iter().enumerate() {
                    let op_next = self.filter.index_filter(index, state, value.len().ok())?;
                    if let Some((next_include, next_exclude)) = op_next {
                        let state = &mut state.scoped_include_exclude(next_include, next_exclude);
                        items.push(item_serializer.to_python(&element, state, extra)?);
                    }
                }
                items.into_py_any(py)
            }
            Err(_) => {
                state.warn_fallback_py(self.get_name(), value)?;
                infer_to_python(value, state, extra)
            }
        }
    }

    fn json_key<'a, 'py>(
        &self,
        key: &'a Bound<'py, PyAny>,
        state: &mut SerializationState<'py>,
        extra: &Extra<'_, 'py>,
    ) -> PyResult<Cow<'a, str>> {
        self.invalid_as_json_key(key, state, extra, Self::EXPECTED_TYPE)
    }

    fn serde_serialize<'py, S: serde::ser::Serializer>(
        &self,
        value: &Bound<'py, PyAny>,
        serializer: S,
        state: &mut SerializationState<'py>,
        extra: &Extra<'_, 'py>,
    ) -> Result<S::Ok, S::Error> {
        match value.downcast::<PyList>() {
            Ok(py_list) => {
                let mut seq = serializer.serialize_seq(Some(py_list.len()))?;
                let item_serializer = self.item_serializer.as_ref();

                for (index, element) in py_list.iter().enumerate() {
                    let op_next = self
                        .filter
                        .index_filter(index, state, Some(py_list.len()))
                        .map_err(py_err_se_err)?;
                    if let Some((next_include, next_exclude)) = op_next {
                        let state = &mut state.scoped_include_exclude(next_include, next_exclude);
                        let item_serialize = PydanticSerializer::new(&element, item_serializer, state, extra);
                        seq.serialize_element(&item_serialize)?;
                    }
                }
                seq.end()
            }
            Err(_) => {
                state.warn_fallback_ser::<S>(self.get_name(), value)?;
                infer_serialize(value, serializer, state, extra)
            }
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn retry_with_lax_check(&self) -> bool {
        self.item_serializer.retry_with_lax_check()
    }
}
