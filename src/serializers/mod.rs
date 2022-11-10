use enum_dispatch::enum_dispatch;
use std::fmt;
use std::fmt::Debug;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

use crate::build_tools::{py_err, py_error_type, SchemaDict};
use crate::PydanticSerializationError;
use serde::Serialize;
use serde_json::ser::PrettyFormatter;

mod list;
mod string;

#[pyclass(module = "pydantic_core._pydantic_core")]
#[derive(Debug, Clone)]
pub struct SchemaSerializer {
    serializer: CombinedSerializer,
    json_size: usize,
}

#[pymethods]
impl SchemaSerializer {
    #[new]
    pub fn py_new(_py: Python, schema: &PyAny, config: Option<&PyDict>) -> PyResult<Self> {
        let serializer = build_serializer(schema, config)?;
        Ok(Self {
            serializer,
            json_size: 1024,
        })
    }

    pub fn to_python(&self, py: Python, value: &PyAny, format: Option<&str>) -> PyResult<PyObject> {
        self.serializer.to_python(py, value, format)
    }

    pub fn to_json(&mut self, py: Python, value: &PyAny, indent: Option<usize>) -> PyResult<PyObject> {
        let writer: Vec<u8> = Vec::with_capacity(self.json_size);

        let serializer = PydanticSerializer::new(value, &self.serializer);

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

        self.json_size = bytes.len();
        let py_bytes = PyBytes::new(py, &bytes);
        Ok(py_bytes.into())
    }
}

pub trait BuildSerializer: Sized {
    const EXPECTED_TYPE: &'static str;

    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<CombinedSerializer>;
}

fn build_specific_serializer<'a, T: BuildSerializer>(
    val_type: &str,
    schema_dict: &'a PyDict,
    config: Option<&'a PyDict>,
) -> PyResult<CombinedSerializer> {
    T::build(schema_dict, config)
        .map_err(|err| py_error_type!("Error building \"{}\" serializer:\n  {}", val_type, err))
}

// macro to build the match statement for validator selection
macro_rules! serializer_match {
    ($type:ident, $dict:ident, $config:ident, $($validator:path,)+) => {
        match $type {
            $(
                <$validator>::EXPECTED_TYPE => build_specific_serializer::<$validator>($type, $dict, $config),
            )+
            _ => {
                return py_err!(r#"Unknown serialization schema type: "{}""#, $type)
            },
        }
    };
}

pub fn build_serializer<'a>(schema: &'a PyAny, config: Option<&'a PyDict>) -> PyResult<CombinedSerializer> {
    let dict: &PyDict = schema.cast_as()?;
    let type_: &str = dict.get_as_req(intern!(schema.py(), "type"))?;
    serializer_match!(
        type_,
        dict,
        config,
        // string type
        string::StrSerializer,
        // list type
        list::ListSerializer,
    )
}

#[derive(Debug, Clone)]
#[enum_dispatch]
pub enum CombinedSerializer {
    Str(string::StrSerializer),
    List(list::ListSerializer),
}

#[enum_dispatch(CombinedSerializer)]
pub trait Serializer: Send + Sync + Clone + Debug {
    fn to_python(
        &self,
        py: Python,
        value: &PyAny,
        format: Option<&str>, // TODO "exclude" arguments
    ) -> PyResult<PyObject>;

    fn serde_serialize<S>(&self, value: &PyAny, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer;
}

struct PydanticSerializer<'py> {
    value: &'py PyAny,
    serializer: &'py CombinedSerializer,
}

impl<'py> PydanticSerializer<'py> {
    fn new(value: &'py PyAny, serializer: &'py CombinedSerializer) -> Self {
        Self { value, serializer }
    }
}

impl<'py> serde::ser::Serialize for PydanticSerializer<'py> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.serializer.serde_serialize(self.value, serializer)
    }
}

fn py_err_to_serde<T: serde::ser::Error, E: fmt::Display>(py_error: E) -> T {
    T::custom(py_error.to_string())
}
