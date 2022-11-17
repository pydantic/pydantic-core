use enum_dispatch::enum_dispatch;
use std::cell::RefCell;
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
    pub fn py_new(schema: &PyDict, config: Option<&PyDict>) -> PyResult<Self> {
        let serializer = CombinedSerializer::build(schema, config)?;
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
        let extra = Extra::new(py, &format);
        let v = match format {
            SerFormat::Json => self.comb_serializer.to_python_json(value, include, exclude, &extra),
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

        let extra = Extra::new(py, &SerFormat::Json);
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

trait BuildSerializer: Sized {
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

#[derive(Debug, Clone)]
#[enum_dispatch]
enum CombinedSerializer {
    Str(string::StrSerializer),
    Int(int::IntSerializer),
    List(list_tuple::ListSerializer),
    Tuple(list_tuple::TupleSerializer),
    Any(any::AnySerializer),
}

// macro to build the match statement for validator selection
macro_rules! serializer_match {
    ($type_:ident, $dict:ident, $config:ident, $($validator:path,)+) => {
        match $type_ {
            $(
                <$validator>::EXPECTED_TYPE => build_specific_serializer::<$validator>($type_, $dict, $config),
            )+
            _ => any::AnySerializer::build($dict, $config),
        }
    };
}

impl BuildSerializer for CombinedSerializer {
    // this value is never used, it's just here to satisfy the trait
    const EXPECTED_TYPE: &'static str = "";

    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        let type_: &str = schema.get_as_req(intern!(schema.py(), "type"))?;
        serializer_match!(
            type_,
            schema,
            config,
            string::StrSerializer,
            int::IntSerializer,
            list_tuple::ListSerializer,
            list_tuple::TupleSerializer,
            any::AnySerializer,
        )
    }
}

#[enum_dispatch(CombinedSerializer)]
trait TypeSerializer: Send + Sync + Clone + Debug + BuildSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
        _extra: &Extra,
    ) -> PyResult<PyObject> {
        Ok(value.into_py(value.py()))
    }

    fn to_python_json(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        self.to_python(value, include, exclude, extra)
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        _extra: &Extra,
    ) -> Result<S::Ok, S::Error>;
}

struct PydanticSerializer<'py> {
    value: &'py PyAny,
    com_serializer: &'py CombinedSerializer,
    extra: &'py Extra<'py>,
    include: Option<&'py PyAny>,
    exclude: Option<&'py PyAny>,
}

impl<'py> PydanticSerializer<'py> {
    fn new(
        value: &'py PyAny,
        com_serializer: &'py CombinedSerializer,
        include: Option<&'py PyAny>,
        exclude: Option<&'py PyAny>,
        extra: &'py Extra<'py>,
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

fn py_err_se_err<T: serde::ser::Error, E: fmt::Display>(py_error: E) -> T {
    T::custom(py_error.to_string())
}

/// Useful things which are passed around by serializers
struct Extra<'a> {
    format: &'a SerFormat,
    ob_type_lookup: &'a ObTypeLookup,
    warnings: CollectWarnings,
}

impl<'a> Extra<'a> {
    fn new(py: Python<'a>, format: &'a SerFormat) -> Self {
        Self {
            format,
            ob_type_lookup: ObTypeLookup::cached(py),
            warnings: CollectWarnings::new(true),
        }
    }
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

#[cfg_attr(debug_assertions, derive(Debug))]
struct CollectWarnings {
    pub active: bool,
    warnings: RefCell<Option<Vec<String>>>,
}

impl CollectWarnings {
    fn new(active: bool) -> Self {
        Self {
            active,
            warnings: RefCell::new(None),
        }
    }

    fn fallback(&self, field_type: &str, value: &PyAny, reason: &str) {
        if self.active {
            let type_name = value.get_type().name().unwrap_or("<unknown python object>");
            let message = format!("Expected `{}` but got `{}` - {}", field_type, type_name, reason);
            let mut op_warnings = self.warnings.borrow_mut();
            if let Some(ref mut warnings) = *op_warnings {
                warnings.push(message);
            } else {
                *op_warnings = Some(vec![message]);
            }
        }
    }

    fn fallback_slow(&self, field_type: &str, value: &PyAny) {
        if self.active {
            self.fallback(field_type, value, "slight slowdown possible");
        }
    }

    fn fallback_filtering(&self, field_type: &str, value: &PyAny) {
        if self.active {
            self.fallback(field_type, value, "filtering via include/exclude unavailable");
        }
    }

    fn final_check(&self, py: Python) -> PyResult<()> {
        if self.active {
            match *self.warnings.borrow() {
                Some(ref warnings) => {
                    let warnings = warnings.iter().map(|w| w.as_str()).collect::<Vec<_>>();
                    let message = format!("Pydantic serializer warnings:\n  {}", warnings.join("\n  "));
                    let user_warning_type = py.import("builtins")?.getattr("UserWarning")?;
                    PyErr::warn(py, user_warning_type, &message, 0)
                }
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }
}
