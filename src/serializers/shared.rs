use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Debug;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use enum_dispatch::enum_dispatch;
use serde::Serialize;
use serde_json::ser::PrettyFormatter;

use crate::build_tools::{py_err, py_error_type, SchemaDict};
use crate::PydanticSerializationError;

use super::any::{fallback_to_python, json_key, ObTypeLookup};
use super::timedelta::TimedeltaMode;

pub(super) trait BuildSerializer: Sized {
    const EXPECTED_TYPE: &'static str;

    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<CombinedSerializer>;
}

/// Build the `CombinedSerializer` enum and implement a `find_serializer` method for it.
macro_rules! combined_serializer {
    (
        enum_only: {$($e_key:ident: $e_serializer:path;)*}
        find_only: {$($builder:path;)*}
        both: {$($b_key:ident: $b_serializer:path;)*}
    ) => {
        #[derive(Debug, Clone)]
        #[enum_dispatch]
        pub(super) enum CombinedSerializer {
            $($e_key($e_serializer),)*
            $($b_key($b_serializer),)*
        }

        impl CombinedSerializer {
            fn find_serializer(
                lookup_type: &str, schema: &PyDict, config: Option<&PyDict>
            ) -> PyResult<Option<CombinedSerializer>> {
                match lookup_type {
                    $(
                        <$b_serializer>::EXPECTED_TYPE => match <$b_serializer>::build(schema, config) {
                            Ok(serializer) => Ok(Some(serializer)),
                            Err(err) => py_err!("Error building `{}` serializer:\n  {}", lookup_type, err),
                        },
                    )*
                    $(
                        <$builder>::EXPECTED_TYPE => match <$builder>::build(schema, config) {
                            Ok(serializer) => Ok(Some(serializer)),
                            Err(err) => py_err!("Error building `{}` serializer:\n  {}", lookup_type, err),
                        },
                    )*
                    _ => Ok(None),
                }
            }
        }

    };
}

combined_serializer! {
    // `enum_only` is for serializers which are not built directly via the `type` key and `find_serializer`
    // but are included in the `CombinedSerializer` enum
    enum_only: {
        // function serializers cannot be defined by type lookup, but must be members of `CombinedSerializer`,
        // hence they're here.
        Function: super::function::FunctionSerializer;
        // `TuplePositionalSerializer` & `TupleVariableSerializer` are created by
        // `TupleBuilder` based on the `mode` parameter.
        TuplePositional: super::tuple::TuplePositionalSerializer;
        TupleVariable: super::tuple::TupleVariableSerializer;
    }
    // `find_only` is for serializers which are built directly via the `type` key and `find_serializer`
    // but aren't actually used for serialization, e.g. their `build` method must return another serializer
    find_only: {
        super::tuple::TupleBuilder;
        super::other::ChainBuilder;
        super::other::FunctionBuilder;
        super::other::CustomErrorBuilder;
        super::literal::LiteralBuildSerializer;
    }
    // `both` means the struct is added to both the `CombinedSerializer` enum and the match statement in
    // `find_serializer` so they can be used via a `type` str.
    both: {
        None: super::simple::NoneSerializer;
        Int: super::simple::IntSerializer;
        Bool: super::simple::BoolSerializer;
        Float: super::simple::FloatSerializer;
        Str: super::string::StrSerializer;
        Bytes: super::bytes::BytesSerializer;
        Datetime: super::datetime_etc::DatetimeSerializer;
        TimeDelta: super::timedelta::TimeDeltaSerializer;
        Date: super::datetime_etc::DateSerializer;
        Time: super::datetime_etc::TimeSerializer;
        List: super::list::ListSerializer;
        Set: super::set_frozenset::SetSerializer;
        FrozenSet: super::set_frozenset::FrozenSetSerializer;
        Dict: super::dict::DictSerializer;
        TypedDict: super::typed_dict::TypedDictSerializer;
        ModelDict: super::new_class::NewClassSerializer;
        Url: super::url::UrlSerializer;
        MultiHostUrl: super::url::MultiHostUrlSerializer;
        Any: super::any::AnySerializer;
        Format: super::format::FunctionSerializer;
        WithDefault: super::with_default::WithDefaultSerializer;
        Json: super::json::JsonSerializer;
    }
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
                // applies to lists tuples and dicts, does not override the main schema `type`
                Some("include-exclude-sequence") | Some("include-exclude-dict") => (),
                // applies specifically to bytes, does not override the main schema `type`
                Some("base64") => (),
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

    fn json_key<'py>(&self, key: &'py PyAny, extra: &Extra) -> PyResult<Cow<'py, str>> {
        json_key(key, extra)
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error>;
}

pub(super) fn py_err_se_err<T: serde::ser::Error, E: fmt::Display>(py_error: E) -> T {
    T::custom(py_error.to_string())
}

pub(super) struct PydanticSerializer<'py> {
    value: &'py PyAny,
    serializer: &'py CombinedSerializer,
    extra: &'py Extra<'py>,
    include: Option<&'py PyAny>,
    exclude: Option<&'py PyAny>,
}

impl<'py> PydanticSerializer<'py> {
    pub(super) fn new(
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

/// Useful things which are passed around by serializers
pub(super) struct Extra<'a> {
    pub mode: &'a SerMode,
    pub ob_type_lookup: &'a ObTypeLookup,
    pub warnings: CollectWarnings,
    pub by_alias: bool,
    pub exclude_unset: bool,
    pub exclude_defaults: bool,
    pub exclude_none: bool,
    pub round_trip: bool,
    pub timedelta_mode: TimedeltaMode,
}

impl<'a> Extra<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        py: Python<'a>,
        mode: &'a SerMode,
        by_alias: Option<bool>,
        exclude_unset: Option<bool>,
        exclude_defaults: Option<bool>,
        exclude_none: Option<bool>,
        round_trip: Option<bool>,
        timedelta_mode: TimedeltaMode,
    ) -> Self {
        Self {
            mode,
            ob_type_lookup: ObTypeLookup::cached(py),
            warnings: CollectWarnings::new(true),
            by_alias: by_alias.unwrap_or(true),
            exclude_unset: exclude_unset.unwrap_or(false),
            exclude_defaults: exclude_defaults.unwrap_or(false),
            exclude_none: exclude_none.unwrap_or(false),
            round_trip: round_trip.unwrap_or(false),
            timedelta_mode,
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
            self.add_warning(format!(
                "Expected `{}` but got `{}` - {}",
                field_type, type_name, reason
            ));
        }
    }

    fn add_warning(&self, message: String) {
        let mut op_warnings = self.warnings.borrow_mut();
        if let Some(ref mut warnings) = *op_warnings {
            warnings.push(message);
        } else {
            *op_warnings = Some(vec![message]);
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

pub(super) fn to_json_bytes(
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
