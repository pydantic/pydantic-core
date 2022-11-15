use enum_dispatch::enum_dispatch;
use std::fmt;
use std::fmt::Debug;

use serde::Serialize;
use serde_json::ser::PrettyFormatter;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

use self::any::ObTypeLookup;
use crate::build_tools::{py_error_type, SchemaDict};
use crate::PydanticSerializationError;

mod any;
mod int;
mod list_tuple;
mod string;

#[pyclass(module = "pydantic_core._pydantic_core")]
#[derive(Debug, Clone)]
pub struct SchemaSerializer {
    comb_serializer: CombinedSerializer,
    json_size: usize,
}

#[pymethods]
impl SchemaSerializer {
    #[new]
    pub fn py_new(schema: &PyAny, config: Option<&PyDict>) -> PyResult<Self> {
        let serializer = build_serializer(schema, config)?;
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
        let format: SerFormat = format.into();
        let ob_type_lookup = ObTypeLookup::cached(py);
        match format {
            SerFormat::Json => self
                .comb_serializer
                .to_python_json(value, ob_type_lookup, include, exclude),
            format => self
                .comb_serializer
                .to_python(value, &format, ob_type_lookup, include, exclude),
        }
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

        let ob_type_lookup = ObTypeLookup::cached(py);
        let serializer = PydanticSerializer::new(value, &self.comb_serializer, ob_type_lookup, include, exclude);

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

    pub fn __repr__(&self) -> String {
        format!("SchemaSerializer(serializer={:#?})", self.comb_serializer)
    }
}

trait BuildSerializer: Sized {
    const EXPECTED_TYPE: &'static str;

    fn build_combined(schema: &PyDict, config: Option<&PyDict>) -> PyResult<CombinedSerializer>;
}

fn build_specific_serializer<'a, T: BuildSerializer>(
    val_type: &str,
    schema_dict: &'a PyDict,
    config: Option<&'a PyDict>,
) -> PyResult<CombinedSerializer> {
    T::build_combined(schema_dict, config)
        .map_err(|err| py_error_type!("Error building \"{}\" serializer:\n  {}", val_type, err))
}

// macro to build the match statement for validator selection
macro_rules! serializer_match {
    ($type_:ident, $dict:ident, $config:ident, $($validator:path,)+) => {
        match $type_ {
            $(
                <$validator>::EXPECTED_TYPE => build_specific_serializer::<$validator>($type_, $dict, $config),
            )+
            _ => any::AnySerializer::build_combined($dict, $config),
        }
    };
}

fn build_serializer<'a>(schema: &'a PyAny, config: Option<&'a PyDict>) -> PyResult<CombinedSerializer> {
    let dict: &PyDict = schema.cast_as()?;
    let type_: &str = dict.get_as_req(intern!(schema.py(), "type"))?;
    serializer_match!(
        type_,
        dict,
        config,
        string::StrSerializer,
        int::IntSerializer,
        list_tuple::ListSerializer,
        list_tuple::TupleSerializer,
        any::AnySerializer,
    )
}

#[derive(Debug, Clone)]
#[enum_dispatch]
enum CombinedSerializer {
    Str(string::StrSerializer),
    Int(int::IntSerializer),
    List(list_tuple::ListSerializer),
    Tuple(list_tuple::TupleSerializer),
    Any(any::AnySerializer),
}

#[enum_dispatch(CombinedSerializer)]
trait TypeSerializer: Send + Sync + Clone + Debug {
    fn to_python(
        &self,
        value: &PyAny,
        _format: &SerFormat,
        _ob_type_lookup: &ObTypeLookup,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
    ) -> PyResult<PyObject> {
        Ok(value.into_py(value.py()))
    }

    fn to_python_json(
        &self,
        value: &PyAny,
        ob_type_lookup: &ObTypeLookup,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
    ) -> PyResult<PyObject> {
        self.to_python(value, &SerFormat::Json, ob_type_lookup, include, exclude)
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        ob_type_lookup: &ObTypeLookup,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
    ) -> Result<S::Ok, S::Error>;
}

struct PydanticSerializer<'py> {
    value: &'py PyAny,
    com_serializer: &'py CombinedSerializer,
    ob_type_lookup: &'py ObTypeLookup,
    include: Option<&'py PyAny>,
    exclude: Option<&'py PyAny>,
}

impl<'py> PydanticSerializer<'py> {
    fn new(
        value: &'py PyAny,
        com_serializer: &'py CombinedSerializer,
        ob_type_lookup: &'py ObTypeLookup,
        include: Option<&'py PyAny>,
        exclude: Option<&'py PyAny>,
    ) -> Self {
        Self {
            value,
            com_serializer,
            ob_type_lookup,
            include,
            exclude,
        }
    }
}

impl<'py> Serialize for PydanticSerializer<'py> {
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.com_serializer
            .serde_serialize(self.value, serializer, self.ob_type_lookup, self.include, self.exclude)
    }
}

fn py_err_se_err<T: serde::ser::Error, E: fmt::Display>(py_error: E) -> T {
    T::custom(py_error.to_string())
}

#[cfg_attr(debug_assertions, derive(Debug))]
enum SerFormat {
    Python,
    Json,
    Other(String),
}

impl From<Option<&str>> for SerFormat {
    fn from(s: Option<&str>) -> Self {
        match s {
            Some("json") => SerFormat::Json,
            Some("python") => SerFormat::Python,
            Some(other) => SerFormat::Other(other.to_string()),
            None => SerFormat::Python,
        }
    }
}
