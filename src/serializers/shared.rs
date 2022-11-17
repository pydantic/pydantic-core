use std::cell::RefCell;
use std::fmt;
use std::fmt::Debug;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use enum_dispatch::enum_dispatch;

use crate::build_tools::{py_error_type, SchemaDict};

use super::any::ObTypeLookup;

pub(super) trait BuildSerializer: Sized {
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
pub(super) enum CombinedSerializer {
    Str(super::string::StrSerializer),
    Int(super::int::IntSerializer),
    List(super::list_tuple::ListSerializer),
    Tuple(super::list_tuple::TupleSerializer),
    Any(super::any::AnySerializer),
}

// macro to build the match statement for validator selection
macro_rules! serializer_match {
    ($type_:ident, $dict:ident, $config:ident, $($validator:path,)+) => {
        match $type_ {
            $(
                <$validator>::EXPECTED_TYPE => build_specific_serializer::<$validator>($type_, $dict, $config),
            )+
            _ => super::any::AnySerializer::build($dict, $config),
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
            super::string::StrSerializer,
            super::int::IntSerializer,
            super::list_tuple::ListSerializer,
            super::list_tuple::TupleSerializer,
            super::any::AnySerializer,
        )
    }
}

#[enum_dispatch(CombinedSerializer)]
pub(super) trait TypeSerializer: Send + Sync + Clone + Debug + BuildSerializer {
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
