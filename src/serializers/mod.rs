use std::fmt::Debug;

use serde::Serialize;
use serde_json::ser::PrettyFormatter;

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

use crate::PydanticSerializationError;

use shared::{BuildSerializer, TypeSerializer};

mod any;
mod function;
mod list_tuple;
mod shared;
mod simple;
mod string;

#[pyclass(module = "pydantic_core._pydantic_core")]
#[derive(Debug, Clone)]
pub struct SchemaSerializer {
    comb_serializer: shared::CombinedSerializer,
    json_size: usize,
}

#[pymethods]
impl SchemaSerializer {
    #[new]
    pub fn py_new(schema: &PyDict, config: Option<&PyDict>) -> PyResult<Self> {
        let serializer = shared::CombinedSerializer::build(schema, config)?;
        Ok(Self {
            comb_serializer: serializer,
            json_size: 1024,
        })
    }

    pub fn to_python(
        &self,
        py: Python,
        value: &PyAny,
        format: Option<&str>,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
    ) -> PyResult<PyObject> {
        let format: shared::SerFormat = format.into();
        let extra = shared::Extra::new(py, &format);
        let v = match format {
            shared::SerFormat::Json => self.comb_serializer.to_python_json(value, include, exclude, &extra),
            _ => self.comb_serializer.to_python(value, include, exclude, &extra),
        }?;
        extra.warnings.final_check(py)?;
        Ok(v)
    }

    pub fn to_json(
        &mut self,
        py: Python,
        value: &PyAny,
        indent: Option<usize>,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
    ) -> PyResult<PyObject> {
        let writer: Vec<u8> = Vec::with_capacity(self.json_size);

        let extra = shared::Extra::new(py, &shared::SerFormat::Json);
        let serializer = PydanticSerializer::new(value, &self.comb_serializer, include, exclude, &extra);

        let bytes = match indent {
            Some(indent_size) => {
                let indent = vec![b' '; indent_size];
                let formatter = PrettyFormatter::with_indent(&indent);
                let mut ser = serde_json::Serializer::with_formatter(writer, formatter);
                serializer
                    .serialize(&mut ser)
                    .map_err(PydanticSerializationError::json_error)?;
                ser.into_inner()
            }
            None => {
                let mut ser = serde_json::Serializer::new(writer);
                serializer
                    .serialize(&mut ser)
                    .map_err(PydanticSerializationError::json_error)?;
                ser.into_inner()
            }
        };

        extra.warnings.final_check(py)?;

        self.json_size = bytes.len();
        let py_bytes = PyBytes::new(py, &bytes);
        Ok(py_bytes.into())
    }

    pub fn __repr__(&self) -> String {
        format!("SchemaSerializer(serializer={:#?})", self.comb_serializer)
    }
}

struct PydanticSerializer<'py> {
    value: &'py PyAny,
    com_serializer: &'py shared::CombinedSerializer,
    extra: &'py shared::Extra<'py>,
    include: Option<&'py PyAny>,
    exclude: Option<&'py PyAny>,
}

impl<'py> PydanticSerializer<'py> {
    fn new(
        value: &'py PyAny,
        com_serializer: &'py shared::CombinedSerializer,
        include: Option<&'py PyAny>,
        exclude: Option<&'py PyAny>,
        extra: &'py shared::Extra<'py>,
    ) -> Self {
        Self {
            value,
            com_serializer,
            include,
            exclude,
            extra,
        }
    }
}

impl<'py> Serialize for PydanticSerializer<'py> {
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.com_serializer
            .serde_serialize(self.value, serializer, self.include, self.exclude, self.extra)
    }
}
