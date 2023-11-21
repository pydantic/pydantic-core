use pyo3::intern2;
use pyo3::prelude::*;
use pyo3::types::PyString;
use pyo3::types::{PyDict, PyList, PyTuple};
use std::borrow::Cow;

use serde::ser::SerializeSeq;

use crate::definitions::DefinitionsBuilder;
use crate::tools::SchemaDict;

use super::any::AnySerializer;
use super::{
    infer_json_key, infer_serialize, infer_to_python, py_err_se_err, BuildSerializer, CombinedSerializer, Extra,
    PydanticSerializer, SchemaFilter, SerMode, TypeSerializer,
};

#[derive(Debug, Clone)]
pub struct TupleVariableSerializer {
    item_serializer: Box<CombinedSerializer>,
    filter: SchemaFilter<usize>,
    name: String,
}

impl BuildSerializer for TupleVariableSerializer {
    const EXPECTED_TYPE: &'static str = "tuple-variable";

    fn build(
        schema: &Py2<'_, PyDict>,
        config: Option<&Py2<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        if let Some("positional") = schema
            .get_as::<Py2<'_, PyString>>(intern2!(py, "mode"))?
            .as_ref()
            .map(|s| s.to_str())
            .transpose()?
        {
            return TuplePositionalSerializer::build(schema, config, definitions);
        }
        let item_serializer = match schema.get_as(intern2!(py, "items_schema"))? {
            Some(items_schema) => CombinedSerializer::build(&items_schema, config, definitions)?,
            None => AnySerializer::build(schema, config, definitions)?,
        };
        let name = format!("tuple[{}, ...]", item_serializer.get_name());
        Ok(Self {
            item_serializer: Box::new(item_serializer),
            filter: SchemaFilter::from_schema(schema)?,
            name,
        }
        .into())
    }
}

impl_py_gc_traverse!(TupleVariableSerializer { item_serializer });

impl TypeSerializer for TupleVariableSerializer {
    fn to_python(
        &self,
        value: &Py2<'_, PyAny>,
        include: Option<&Py2<'_, PyAny>>,
        exclude: Option<&Py2<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        match value.downcast::<PyTuple>() {
            Ok(py_tuple) => {
                let py = value.py();
                let item_serializer = self.item_serializer.as_ref();

                let mut items = Vec::with_capacity(py_tuple.len());
                for (index, element) in py_tuple.iter().enumerate() {
                    let op_next = self
                        .filter
                        .index_filter(index, include, exclude, Some(py_tuple.len()))?;
                    if let Some((next_include, next_exclude)) = op_next {
                        items.push(item_serializer.to_python(
                            &element,
                            next_include.as_ref(),
                            next_exclude.as_ref(),
                            extra,
                        )?);
                    }
                }
                match extra.mode {
                    SerMode::Json => Ok(PyList::new2(py, items).into_py(py)),
                    _ => Ok(PyTuple::new2(py, items).into_py(py)),
                }
            }
            Err(_) => {
                extra.warnings.on_fallback_py(&self.name, value, extra)?;
                infer_to_python(value, include, exclude, extra)
            }
        }
    }

    fn json_key<'py>(&self, key: &Py2<'py, PyAny>, extra: &Extra) -> PyResult<Cow<'py, str>> {
        match key.downcast::<PyTuple>() {
            Ok(py_tuple) => {
                let item_serializer = self.item_serializer.as_ref();

                let mut key_builder = KeyBuilder::new();
                for element in py_tuple {
                    key_builder.push(&item_serializer.json_key(&element, extra)?);
                }
                Ok(Cow::Owned(key_builder.finish()))
            }
            Err(_) => {
                extra.warnings.on_fallback_py(&self.name, key, extra)?;
                infer_json_key(key, extra)
            }
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Py2<'_, PyAny>,
        serializer: S,
        include: Option<&Py2<'_, PyAny>>,
        exclude: Option<&Py2<'_, PyAny>>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        match value.downcast::<PyTuple>() {
            Ok(py_tuple) => {
                let item_serializer = self.item_serializer.as_ref();

                let mut seq = serializer.serialize_seq(Some(py_tuple.len()))?;
                for (index, element) in py_tuple.iter().enumerate() {
                    let op_next = self
                        .filter
                        .index_filter(index, include, exclude, Some(py_tuple.len()))
                        .map_err(py_err_se_err)?;
                    if let Some((next_include, next_exclude)) = op_next {
                        let item_serialize = PydanticSerializer::new(
                            &element,
                            item_serializer,
                            next_include.as_ref(),
                            next_exclude.as_ref(),
                            extra,
                        );
                        seq.serialize_element(&item_serialize)?;
                    }
                }
                seq.end()
            }
            Err(_) => {
                extra.warnings.on_fallback_ser::<S>(&self.name, value, extra)?;
                infer_serialize(value, serializer, include, exclude, extra)
            }
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct TuplePositionalSerializer {
    items_serializers: Vec<CombinedSerializer>,
    extra_serializer: Box<CombinedSerializer>,
    filter: SchemaFilter<usize>,
    name: String,
}

impl BuildSerializer for TuplePositionalSerializer {
    const EXPECTED_TYPE: &'static str = "tuple-positional";

    fn build(
        schema: &Py2<'_, PyDict>,
        config: Option<&Py2<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let items: Py2<'_, PyList> = schema.get_as_req(intern2!(py, "items_schema"))?;

        let extra_serializer = match schema.get_as(intern2!(py, "extras_schema"))? {
            Some(extras_schema) => CombinedSerializer::build(&extras_schema, config, definitions)?,
            None => AnySerializer::build(schema, config, definitions)?,
        };
        let items_serializers: Vec<CombinedSerializer> = items
            .iter()
            .map(|item| CombinedSerializer::build(item.downcast()?, config, definitions))
            .collect::<PyResult<_>>()?;

        let descr = items_serializers
            .iter()
            .map(TypeSerializer::get_name)
            .collect::<Vec<_>>()
            .join(", ");
        Ok(Self {
            items_serializers,
            extra_serializer: Box::new(extra_serializer),
            filter: SchemaFilter::from_schema(schema)?,
            name: format!("tuple[{descr}]"),
        }
        .into())
    }
}

impl_py_gc_traverse!(TuplePositionalSerializer {
    items_serializers,
    extra_serializer
});

impl TypeSerializer for TuplePositionalSerializer {
    fn to_python(
        &self,
        value: &Py2<'_, PyAny>,
        include: Option<&Py2<'_, PyAny>>,
        exclude: Option<&Py2<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        match value.downcast::<PyTuple>() {
            Ok(py_tuple) => {
                let py = value.py();

                let mut py_tuple_iter = py_tuple.iter();
                let mut items = Vec::with_capacity(py_tuple.len());
                for (index, serializer) in self.items_serializers.iter().enumerate() {
                    let element = match py_tuple_iter.next() {
                        Some(value) => value,
                        None => break,
                    };
                    let op_next = self
                        .filter
                        .index_filter(index, include, exclude, Some(py_tuple.len()))?;
                    if let Some((next_include, next_exclude)) = op_next {
                        items.push(serializer.to_python(
                            &element,
                            next_include.as_ref(),
                            next_exclude.as_ref(),
                            extra,
                        )?);
                    }
                }
                let expected_length = self.items_serializers.len();
                let extra_serializer = self.extra_serializer.as_ref();
                for (index2, element) in py_tuple_iter.enumerate() {
                    let index = index2 + expected_length;
                    let op_next = self
                        .filter
                        .index_filter(index, include, exclude, Some(py_tuple.len()))?;
                    if let Some((next_include, next_exclude)) = op_next {
                        items.push(extra_serializer.to_python(
                            &element,
                            next_include.as_ref(),
                            next_exclude.as_ref(),
                            extra,
                        )?);
                    }
                }

                match extra.mode {
                    SerMode::Json => Ok(PyList::new2(py, items).into_py(py)),
                    _ => Ok(PyTuple::new2(py, items).into_py(py)),
                }
            }
            Err(_) => {
                extra.warnings.on_fallback_py(&self.name, value, extra)?;
                infer_to_python(value, include, exclude, extra)
            }
        }
    }

    fn json_key<'py>(&self, key: &Py2<'py, PyAny>, extra: &Extra) -> PyResult<Cow<'py, str>> {
        match key.downcast::<PyTuple>() {
            Ok(py_tuple) => {
                let mut py_tuple_iter = py_tuple.iter();

                let mut key_builder = KeyBuilder::new();
                for serializer in &self.items_serializers {
                    let element = match py_tuple_iter.next() {
                        Some(value) => value,
                        None => break,
                    };
                    key_builder.push(&serializer.json_key(&element, extra)?);
                }
                let extra_serializer = self.extra_serializer.as_ref();
                for element in py_tuple_iter {
                    key_builder.push(&extra_serializer.json_key(&element, extra)?);
                }
                Ok(Cow::Owned(key_builder.finish()))
            }
            Err(_) => {
                extra.warnings.on_fallback_py(&self.name, key, extra)?;
                infer_json_key(key, extra)
            }
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Py2<'_, PyAny>,
        serializer: S,
        include: Option<&Py2<'_, PyAny>>,
        exclude: Option<&Py2<'_, PyAny>>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        match value.downcast::<PyTuple>() {
            Ok(py_tuple) => {
                let mut py_tuple_iter = py_tuple.iter();
                let mut seq = serializer.serialize_seq(Some(py_tuple.len()))?;
                for (index, serializer) in self.items_serializers.iter().enumerate() {
                    let element = match py_tuple_iter.next() {
                        Some(value) => value,
                        None => break,
                    };
                    let op_next = self
                        .filter
                        .index_filter(index, include, exclude, Some(py_tuple.len()))
                        .map_err(py_err_se_err)?;
                    if let Some((next_include, next_exclude)) = op_next {
                        let item_serialize = PydanticSerializer::new(
                            &element,
                            serializer,
                            next_include.as_ref(),
                            next_exclude.as_ref(),
                            extra,
                        );
                        seq.serialize_element(&item_serialize)?;
                    }
                }

                let expected_length = self.items_serializers.len();
                let extra_serializer = self.extra_serializer.as_ref();
                for (index2, element) in py_tuple_iter.enumerate() {
                    let index = index2 + expected_length;
                    let op_next = self
                        .filter
                        .index_filter(index, include, exclude, Some(py_tuple.len()))
                        .map_err(py_err_se_err)?;
                    if let Some((next_include, next_exclude)) = op_next {
                        let item_serialize = PydanticSerializer::new(
                            &element,
                            extra_serializer,
                            next_include.as_ref(),
                            next_exclude.as_ref(),
                            extra,
                        );
                        seq.serialize_element(&item_serialize)?;
                    }
                }

                seq.end()
            }
            Err(_) => {
                extra.warnings.on_fallback_ser::<S>(&self.name, value, extra)?;
                infer_serialize(value, serializer, include, exclude, extra)
            }
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

pub(crate) struct KeyBuilder {
    key: String,
    first: bool,
}

impl KeyBuilder {
    pub fn new() -> Self {
        Self {
            key: String::with_capacity(31),
            first: true,
        }
    }

    pub fn push(&mut self, key: &str) {
        if self.first {
            self.first = false;
        } else {
            self.key.push(',');
        }
        self.key.push_str(key);
    }

    pub fn finish(self) -> String {
        self.key
    }
}
