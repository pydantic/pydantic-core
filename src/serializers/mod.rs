use std::fmt::Debug;

use serde::Serialize;
use serde_json::ser::PrettyFormatter;

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

use crate::{PydanticSerializationError, SchemaValidator};

use shared::{BuildSerializer, CombinedSerializer, Extra, SerMode, TypeSerializer};

mod any;
mod bytes;
mod datetime_etc;
mod dict;
mod format;
mod function;
mod include_exclude;
mod json;
mod list;
mod new_class;
mod set_frozenset;
mod shared;
mod simple;
mod string;
mod timedelta;
mod tuple;
mod typed_dict;
mod url;
mod with_default;

#[pyclass(module = "pydantic_core._pydantic_core")]
#[derive(Debug, Clone)]
pub struct SchemaSerializer {
    serializer: CombinedSerializer,
    json_size: usize,
    timedelta_mode: timedelta::TimedeltaMode,
}

#[pymethods]
impl SchemaSerializer {
    #[new]
    pub fn py_new(py: Python, schema: &PyDict, config: Option<&PyDict>) -> PyResult<Self> {
        let schema = SchemaValidator::validate_schema(py, schema)?.cast_as()?;
        let serializer = CombinedSerializer::build(schema, config)?;
        Ok(Self {
            serializer,
            json_size: 1024,
            timedelta_mode: timedelta::TimedeltaMode::from_config(config)?,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn to_python(
        &self,
        py: Python,
        value: &PyAny,
        mode: Option<&str>,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        by_alias: Option<bool>,
        exclude_unset: Option<bool>,
        exclude_defaults: Option<bool>,
        exclude_none: Option<bool>,
        round_trip: Option<bool>,
    ) -> PyResult<PyObject> {
        let mode: SerMode = mode.into();
        let extra = Extra::new(
            py,
            &mode,
            by_alias,
            exclude_unset,
            exclude_defaults,
            exclude_none,
            round_trip,
            self.timedelta_mode,
        );
        let v = self.serializer.to_python(value, include, exclude, &extra)?;
        extra.warnings.final_check(py)?;
        Ok(v)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn to_json(
        &mut self,
        py: Python,
        value: &PyAny,
        indent: Option<usize>,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        by_alias: Option<bool>,
        exclude_unset: Option<bool>,
        exclude_defaults: Option<bool>,
        exclude_none: Option<bool>,
        round_trip: Option<bool>,
    ) -> PyResult<PyObject> {
        let mode = SerMode::Json;
        let extra = Extra::new(
            py,
            &mode,
            by_alias,
            exclude_unset,
            exclude_defaults,
            exclude_none,
            round_trip,
            self.timedelta_mode,
        );
        let bytes = to_json_bytes(
            value,
            &self.serializer,
            include,
            exclude,
            &extra,
            indent,
            self.json_size,
        )?;

        extra.warnings.final_check(py)?;

        self.json_size = bytes.len();
        let py_bytes = PyBytes::new(py, &bytes);
        Ok(py_bytes.into())
    }

    pub fn __repr__(&self) -> String {
        format!("SchemaSerializer(serializer={:#?})", self.serializer)
    }
}

struct PydanticSerializer<'py> {
    value: &'py PyAny,
    serializer: &'py CombinedSerializer,
    extra: &'py Extra<'py>,
    include: Option<&'py PyAny>,
    exclude: Option<&'py PyAny>,
}

impl<'py> PydanticSerializer<'py> {
    fn new(
        value: &'py PyAny,
        serializer: &'py CombinedSerializer,
        include: Option<&'py PyAny>,
        exclude: Option<&'py PyAny>,
        extra: &'py Extra<'py>,
    ) -> Self {
        Self {
            value,
            serializer,
            include,
            exclude,
            extra,
        }
    }
}

impl<'py> Serialize for PydanticSerializer<'py> {
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.serializer
            .serde_serialize(self.value, serializer, self.include, self.exclude, self.extra)
    }
}

fn to_json_bytes(
    value: &PyAny,
    serializer: &CombinedSerializer,
    include: Option<&PyAny>,
    exclude: Option<&PyAny>,
    extra: &Extra,
    indent: Option<usize>,
    json_size: usize,
) -> PyResult<Vec<u8>> {
    let serializer = PydanticSerializer::new(value, serializer, include, exclude, extra);

    let writer: Vec<u8> = Vec::with_capacity(json_size);
    let bytes = match indent {
        Some(indent) => {
            let indent = vec![b' '; indent];
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
    Ok(bytes)
}
