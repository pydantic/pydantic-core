use std::cell::RefCell;
use std::fmt;
use std::fmt::Debug;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use enum_dispatch::enum_dispatch;

use crate::build_tools::{py_err, py_error_type, SchemaDict};

use super::any::{fallback_to_python, ObTypeLookup};

pub(super) trait BuildSerializer: Sized {
    const EXPECTED_TYPE: &'static str;

    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<CombinedSerializer>;
}

/// `s_match` is used within `combined_serializer` to exclude `enum_only` arguments from the `find_serializer`
/// match statement.
macro_rules! s_match {
    (both, $validator:path, $schema:ident, $config:ident, $lookup_type:ident) => {
        match <$validator>::build($schema, $config) {
            Ok(serializer) => Ok(Some(serializer)),
            Err(err) => py_err!("Error building `{}` serializer:\n  {}", $lookup_type, err),
        }
    };
    (enum_only, $validator:path, $schema:ident, $config:ident, $lookup_type:ident) => {
        Ok(None)
    };
}

/// Build the `CombinedSerializer` enum and implement a `find_serializer` method for it.
macro_rules! combined_serializer {
    ($($classifier:ident: $key:ident, $validator:path;)+) => {
        #[derive(Debug, Clone)]
        #[enum_dispatch]
        pub(super) enum CombinedSerializer {
            $( $key($validator), )+
        }

        impl CombinedSerializer {
            fn find_serializer(
                lookup_type: &str, schema: &PyDict, config: Option<&PyDict>
            ) -> PyResult<Option<CombinedSerializer>> {
                match lookup_type {
                    $(
                        <$validator>::EXPECTED_TYPE => s_match!($classifier, $validator, schema, config, lookup_type),
                    )*
                    _ => Ok(None),
                }
            }
        }

    };
}

combined_serializer! {
    // function serializers can't be defined by type lookup, but must be members of `CombinedSerializer`,
    // hence they're `enum_only` here.
    enum_only: Function, super::function::FunctionSerializer;
    // both means the struct is added to both the `CombinedSerializer` enum the match statement in `find_serializer`
    // so they can be used via a `type` str.
    both: None, super::simple::NoneSerializer;
    both: Int, super::simple::IntSerializer;
    both: Bool, super::simple::BoolSerializer;
    both: Float, super::simple::FloatSerializer;
    both: Str, super::string::StrSerializer;
    both: List, super::list_tuple::ListSerializer;
    both: Tuple, super::list_tuple::TupleSerializer;
    both: Any, super::any::AnySerializer;
    both: Format, super::format::FunctionSerializer;
}

impl BuildSerializer for CombinedSerializer {
    // this value is never used, it's just here to satisfy the trait
    const EXPECTED_TYPE: &'static str = "";

    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let type_key = intern!(py, "type");

        if let Some(ser) = schema.get_as::<&PyDict>(intern!(py, "serialization"))? {
            let op_ser_type: Option<&str> = ser.get_as(intern!(py, "type"))?;
            match op_ser_type {
                Some("function") => {
                    // `function` is a special case, not inclued in `find_serializer` since it means something different
                    // in `schema.type`
                    return super::function::FunctionSerializer::build(ser, config)
                        .map_err(|err| py_error_type!("Error building `function` serializer:\n  {}", err));
                }
                Some(ser_type) => {
                    // otherwise if `schema.serialization.type` is defined, use that with `find_serializer`
                    // instead of `schema.type`. In this case it's an error if a serializer isn't found.
                    return match Self::find_serializer(ser_type, schema, config)? {
                        Some(serializer) => Ok(serializer),
                        None => py_err!("Unknown serialization schema type: `{}`", ser_type),
                    };
                }
                // if `schema.serialization.type` is None, fall back to `schema.type`
                None => (),
            };
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
        extra: &Extra,
    ) -> PyResult<PyObject> {
        fallback_to_python(value, extra)
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
    pub mode: &'a SerMode,
    pub ob_type_lookup: &'a ObTypeLookup,
    pub warnings: CollectWarnings,
}

impl<'a> Extra<'a> {
    pub(super) fn new(py: Python<'a>, mode: &'a SerMode) -> Self {
        Self {
            mode,
            ob_type_lookup: ObTypeLookup::cached(py),
            warnings: CollectWarnings::new(true),
        }
    }
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub(super) enum SerMode {
    Python,
    Json,
    Other(String),
}

impl From<Option<&str>> for SerMode {
    fn from(s: Option<&str>) -> Self {
        match s {
            Some("json") => SerMode::Json,
            Some("python") => SerMode::Python,
            Some(other) => SerMode::Other(other.to_string()),
            None => SerMode::Python,
        }
    }
}

impl ToPyObject for SerMode {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            SerMode::Python => intern!(py, "python").to_object(py),
            SerMode::Json => intern!(py, "json").to_object(py),
            SerMode::Other(s) => s.to_object(py),
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
