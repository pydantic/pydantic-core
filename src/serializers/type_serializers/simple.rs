use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::IntoPyObjectExt;

use std::borrow::Cow;

use serde::Serialize;

use crate::PydanticSerializationUnexpectedValue;
use crate::{definitions::DefinitionsBuilder, input::Int};

use super::{
    infer_json_key, infer_serialize, infer_to_python, BuildSerializer, CombinedSerializer, Extra, IsType, ObType,
    SerCheck, SerMode, TypeSerializer,
};

#[derive(Debug)]
pub struct NoneSerializer;

impl BuildSerializer for NoneSerializer {
    const EXPECTED_TYPE: &'static str = "none";

    fn build(
        _schema: &Bound<'_, PyDict>,
        _config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        Ok(Self {}.into())
    }
}

pub(crate) fn none_json_key() -> PyResult<Cow<'static, str>> {
    Ok(Cow::Borrowed("None"))
}

impl_py_gc_traverse!(NoneSerializer {});

impl TypeSerializer for NoneSerializer {
    fn to_python(
        &self,
        value: &Bound<'_, PyAny>,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let py = value.py();
        match extra.ob_type_lookup.is_type(value, ObType::None) {
            IsType::Exact => Ok(py.None()),
            // I don't think subclasses of None can exist
            _ => {
                extra.warnings.on_fallback_py(self.get_name(), value, extra)?;
                infer_to_python(value, include, exclude, extra)
            }
        }
    }

    fn json_key<'a>(&self, key: &'a Bound<'_, PyAny>, extra: &Extra) -> PyResult<Cow<'a, str>> {
        match extra.ob_type_lookup.is_type(key, ObType::None) {
            IsType::Exact => none_json_key(),
            _ => {
                extra.warnings.on_fallback_py(self.get_name(), key, extra)?;
                infer_json_key(key, extra)
            }
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Bound<'_, PyAny>,
        serializer: S,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        match extra.ob_type_lookup.is_type(value, ObType::None) {
            IsType::Exact => serializer.serialize_none(),
            _ => {
                extra.warnings.on_fallback_ser::<S>(self.get_name(), value, extra)?;
                infer_serialize(value, serializer, include, exclude, extra)
            }
        }
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}

macro_rules! build_simple_serializer {
    ($struct_name:ident, $expected_type:literal, $rust_type:ty, $ob_type:expr, $key_method:ident, $subtypes_allowed:expr) => {
        #[derive(Debug)]
        pub struct $struct_name;

        impl $struct_name {
            pub fn new() -> Self {
                Self {}
            }
        }

        impl BuildSerializer for $struct_name {
            const EXPECTED_TYPE: &'static str = $expected_type;

            fn build(
                _schema: &Bound<'_, PyDict>,
                _config: Option<&Bound<'_, PyDict>>,
                _definitions: &mut DefinitionsBuilder<CombinedSerializer>,
            ) -> PyResult<CombinedSerializer> {
                Ok(Self::new().into())
            }
        }

        impl_py_gc_traverse!($struct_name {});

        impl TypeSerializer for $struct_name {
            fn to_python(
                &self,
                value: &Bound<'_, PyAny>,
                include: Option<&Bound<'_, PyAny>>,
                exclude: Option<&Bound<'_, PyAny>>,
                extra: &Extra,
            ) -> PyResult<PyObject> {
                let py = value.py();
                match extra.ob_type_lookup.is_type(value, $ob_type) {
                    IsType::Exact => Ok(value.clone().unbind()),
                    IsType::Subclass => match extra.check {
                        SerCheck::Strict => Err(PydanticSerializationUnexpectedValue::new_from_msg(None).to_py_err()),
                        SerCheck::Lax | SerCheck::None => match extra.mode {
                            SerMode::Json => value.extract::<$rust_type>()?.into_py_any(py),
                            _ => infer_to_python(value, include, exclude, extra),
                        },
                    },
                    IsType::False => {
                        extra.warnings.on_fallback_py(self.get_name(), value, extra)?;
                        infer_to_python(value, include, exclude, extra)
                    }
                }
            }

            fn json_key<'a>(&self, key: &'a Bound<'_, PyAny>, extra: &Extra) -> PyResult<Cow<'a, str>> {
                match extra.ob_type_lookup.is_type(key, $ob_type) {
                    IsType::Exact | IsType::Subclass => $key_method(key),
                    IsType::False => {
                        extra.warnings.on_fallback_py(self.get_name(), key, extra)?;
                        infer_json_key(key, extra)
                    }
                }
            }

            fn serde_serialize<S: serde::ser::Serializer>(
                &self,
                value: &Bound<'_, PyAny>,
                serializer: S,
                include: Option<&Bound<'_, PyAny>>,
                exclude: Option<&Bound<'_, PyAny>>,
                extra: &Extra,
            ) -> Result<S::Ok, S::Error> {
                match value.extract::<$rust_type>() {
                    Ok(v) => v.serialize(serializer),
                    Err(_) => {
                        extra
                            .warnings
                            .on_fallback_ser::<S>(self.get_name(), value, extra)?;
                        infer_serialize(value, serializer, include, exclude, extra)
                    }
                }
            }

            fn get_name(&self) -> &str {
                Self::EXPECTED_TYPE
            }

            fn retry_with_lax_check(&self) -> bool {
                $subtypes_allowed
            }
        }
    };
}

pub(crate) fn to_str_json_key<'a>(key: &'a Bound<'_, PyAny>) -> PyResult<Cow<'a, str>> {
    Ok(Cow::Owned(key.str()?.to_string_lossy().into_owned()))
}

build_simple_serializer!(IntSerializer, "int", Int, ObType::Int, to_str_json_key, true);

pub(crate) fn bool_json_key<'a>(key: &'a Bound<'_, PyAny>) -> PyResult<Cow<'a, str>> {
    let v = if key.is_truthy().unwrap_or(false) {
        "true"
    } else {
        "false"
    };
    Ok(Cow::Borrowed(v))
}

build_simple_serializer!(BoolSerializer, "bool", bool, ObType::Bool, bool_json_key, false);
