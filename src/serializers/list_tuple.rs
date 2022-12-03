use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};

use serde::ser::SerializeSeq;

use crate::build_tools::SchemaDict;

use super::any::{fallback_serialize, fallback_to_python, AnySerializer};
use super::include_exclude::SchemaIncEx;
use super::shared::{py_err_se_err, BuildSerializer, CombinedSerializer, Extra, SerMode, TypeSerializer};
use super::PydanticSerializer;

macro_rules! build_serializer {
    ($struct_name:ident, $expected_type:literal) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name {
            item_serializer: Box<CombinedSerializer>,
            inc_ex: SchemaIncEx<usize>,
        }

        impl BuildSerializer for $struct_name {
            const EXPECTED_TYPE: &'static str = $expected_type;

            fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
                let py = schema.py();
                let item_serializer = match schema.get_as::<&PyDict>(intern!(py, "items_schema"))? {
                    Some(items_schema) => CombinedSerializer::build(items_schema, config)?,
                    None => AnySerializer::build(schema, config)?,
                };
                let inc_ex = match schema.get_as::<&PyDict>(intern!(py, "serialization"))? {
                    Some(ser) => {
                        let include = ser.get_item(intern!(py, "include"));
                        let exclude = ser.get_item(intern!(py, "exclude"));
                        SchemaIncEx::from_ints(include, exclude)?
                    }
                    None => SchemaIncEx::default(),
                };
                Ok(Self {
                    item_serializer: Box::new(item_serializer),
                    inc_ex,
                }
                .into())
            }
        }

        impl $struct_name {
            fn include_or_exclude<'s, 'py>(
                &'s self,
                py: Python<'py>,
                index: usize,
                include: Option<&'py PyAny>,
                exclude: Option<&'py PyAny>,
            ) -> PyResult<Option<(Option<&'py PyAny>, Option<&'py PyAny>)>> {
                let py_key = index.to_object(py);
                self.inc_ex
                    .include_or_exclude(py_key.as_ref(py), index, include, exclude)
            }
        }
    };
}

build_serializer!(ListSerializer, "list");

impl TypeSerializer for ListSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        match value.cast_as::<PyList>() {
            Ok(py_list) => {
                let py = value.py();
                let item_serializer = self.item_serializer.as_ref();

                let mut items = Vec::with_capacity(py_list.len());
                for (index, element) in py_list.iter().enumerate() {
                    if let Some((next_include, next_exclude)) = self.include_or_exclude(py, index, include, exclude)? {
                        items.push(item_serializer.to_python(element, next_include, next_exclude, extra)?);
                    }
                }
                Ok(items.into_py(py))
            }
            Err(_) => {
                extra.warnings.fallback_filtering(Self::EXPECTED_TYPE, value);
                fallback_to_python(value, extra)
            }
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        match value.cast_as::<PyList>() {
            Ok(py_list) => {
                let mut seq = serializer.serialize_seq(Some(py_list.len()))?;
                let item_serializer = self.item_serializer.as_ref();

                for (index, value) in py_list.iter().enumerate() {
                    if let Some((next_include, next_exclude)) = self
                        .include_or_exclude(value.py(), index, include, exclude)
                        .map_err(py_err_se_err)?
                    {
                        let item_serialize =
                            PydanticSerializer::new(value, item_serializer, next_include, next_exclude, extra);
                        seq.serialize_element(&item_serialize)?;
                    }
                }
                seq.end()
            }
            Err(_) => {
                extra.warnings.fallback_filtering(Self::EXPECTED_TYPE, value);
                fallback_serialize(value, serializer, extra)
            }
        }
    }
}

build_serializer!(TupleSerializer, "tuple");

impl TypeSerializer for TupleSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        match value.cast_as::<PyTuple>() {
            Ok(py_tuple) => {
                let py = value.py();
                let item_serializer = self.item_serializer.as_ref();

                let mut items = Vec::with_capacity(py_tuple.len());
                for (index, element) in py_tuple.iter().enumerate() {
                    if let Some((next_include, next_exclude)) = self.include_or_exclude(py, index, include, exclude)? {
                        items.push(item_serializer.to_python(element, next_include, next_exclude, extra)?);
                    }
                }
                match extra.mode {
                    SerMode::Json => Ok(PyList::new(py, items).into_py(py)),
                    _ => Ok(PyTuple::new(py, items).into_py(py)),
                }
            }
            Err(_) => {
                extra.warnings.fallback_filtering(Self::EXPECTED_TYPE, value);
                fallback_to_python(value, extra)
            }
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        match value.cast_as::<PyTuple>() {
            Ok(py_tuple) => {
                let py_tuple: &PyTuple = py_tuple.cast_as().map_err(py_err_se_err)?;
                let item_serializer = self.item_serializer.as_ref();

                let mut seq = serializer.serialize_seq(Some(py_tuple.len()))?;
                for (index, value) in py_tuple.iter().enumerate() {
                    if let Some((next_include, next_exclude)) = self
                        .include_or_exclude(value.py(), index, include, exclude)
                        .map_err(py_err_se_err)?
                    {
                        let item_serialize =
                            PydanticSerializer::new(value, item_serializer, next_include, next_exclude, extra);
                        seq.serialize_element(&item_serialize)?;
                    }
                }
                seq.end()
            }
            Err(_) => {
                extra.warnings.fallback_filtering(Self::EXPECTED_TYPE, value);
                fallback_serialize(value, serializer, extra)
            }
        }
    }
}
