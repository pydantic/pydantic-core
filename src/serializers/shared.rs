use std::cell::RefCell;
use std::fmt;
use std::fmt::Debug;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use enum_dispatch::enum_dispatch;

use crate::build_tools::{py_err, py_error_type, SchemaDict};

use super::any::ObTypeLookup;

pub(super) trait BuildSerializer: Sized {
    const EXPECTED_TYPE: &'static str;

    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<CombinedSerializer>;
}

macro_rules! combined_serializer {
    ($($key:ident: $validator:path,)+) => {
        #[derive(Debug, Clone)]
        #[enum_dispatch]
        pub(super) enum CombinedSerializer {
            // function serializers can't be defined by type lookup, but are members of `CombinedSerializer`,
            // hence defined here
            Function(super::function::FunctionSerializer),
            $( $key($validator), )+
        }

        impl CombinedSerializer {
            fn find_serializer(lookup_type: &str, schema: &PyDict, config: Option<&PyDict>) -> PyResult<Option<CombinedSerializer>> {
                match lookup_type {
                    $(
                        <$validator>::EXPECTED_TYPE => match <$validator>::build(schema, config) {
                            Ok(serializer) => Ok(Some(serializer)),
                            Err(err) => py_err!("Error building `{}` serializer:\n  {}", lookup_type, err),
                        },
                    )+
                    _ => Ok(None),
                }
            }
        }

    };
}

combined_serializer! {
    Str: super::string::StrSerializer,
    Int: super::int::IntSerializer,
    List: super::list_tuple::ListSerializer,
    Tuple: super::list_tuple::TupleSerializer,
    Any: super::any::AnySerializer,
}

impl BuildSerializer for CombinedSerializer {
    // this value is never used, it's just here to satisfy the trait
    const EXPECTED_TYPE: &'static str = "";

    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let type_key = intern!(py, "type");
        if let Some(ser) = schema.get_as::<&PyDict>(intern!(py, "serialization"))? {
            if ser.contains(intern!(py, "function")).unwrap_or(false) {
                // function is defined in `serialization` dict, use a function serializer
                return super::function::FunctionSerializer::build(ser, config)
                    .map_err(|err| py_error_type!("Error building `function` serializer:\n  {}", err));
            } else if let Some(ser_type) = ser.get_as::<&str>(type_key)? {
                // `type` is defined in `serialization` dict but not function, we use `ser_type` with `find_serializer`
                return match Self::find_serializer(ser_type, schema, config)? {
                    Some(serializer) => Ok(serializer),
                    None => py_err!("Unknown serialization schema type: `{}`", ser_type),
                };
            }
        }
        let type_: &str = schema.get_as_req(type_key)?;
        match Self::find_serializer(type_, schema, config)? {
            Some(serializer) => Ok(serializer),
            None => super::any::AnySerializer::build(schema, config),
        }
    }
}

#[enum_dispatch(CombinedSerializer)]
pub(super) trait TypeSerializer: Send + Sync + Clone + Debug {
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

pub(super) fn py_err_se_err<T: serde::ser::Error, E: fmt::Display>(py_error: E) -> T {
    T::custom(py_error.to_string())
}

/// Useful things which are passed around by serializers
pub(super) struct Extra<'a> {
    pub format: &'a SerFormat,
    pub ob_type_lookup: &'a ObTypeLookup,
    pub warnings: CollectWarnings,
}

impl<'a> Extra<'a> {
    pub(super) fn new(py: Python<'a>, format: &'a SerFormat) -> Self {
        Self {
            format,
            ob_type_lookup: ObTypeLookup::cached(py),
            warnings: CollectWarnings::new(true),
        }
    }
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub(super) enum SerFormat {
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

impl ToPyObject for SerFormat {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            SerFormat::Python => intern!(py, "python").to_object(py),
            SerFormat::Json => intern!(py, "json").to_object(py),
            SerFormat::Other(s) => s.to_object(py),
        }
    }
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub(super) struct CollectWarnings {
    active: bool,
    warnings: RefCell<Option<Vec<String>>>,
}

impl CollectWarnings {
    pub(super) fn new(active: bool) -> Self {
        Self {
            active,
            warnings: RefCell::new(None),
        }
    }

    pub(super) fn fallback_slow(&self, field_type: &str, value: &PyAny) {
        if self.active {
            self.fallback(field_type, value, "slight slowdown possible");
        }
    }

    pub(super) fn fallback_filtering(&self, field_type: &str, value: &PyAny) {
        if self.active {
            self.fallback(field_type, value, "filtering via include/exclude unavailable");
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

    pub(super) fn final_check(&self, py: Python) -> PyResult<()> {
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
