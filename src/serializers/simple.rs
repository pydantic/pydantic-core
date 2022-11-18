use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::Serialize;

use super::any::fallback_serialize;
use super::shared::{BuildSerializer, CombinedSerializer, Extra, TypeSerializer};

#[derive(Debug, Clone)]
pub struct NoneSerializer;

impl BuildSerializer for NoneSerializer {
    const EXPECTED_TYPE: &'static str = "none";

    fn build(_schema: &PyDict, _config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        Ok(Self {}.into())
    }
}

impl TypeSerializer for NoneSerializer {
    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        match value.is_none() {
            true => serializer.serialize_none(),
            false => {
                extra.warnings.fallback_slow(Self::EXPECTED_TYPE, value);
                fallback_serialize(value, serializer, extra.ob_type_lookup)
            }
        }
    }
}

macro_rules! build_simple_serializer {
    ($struct_name:ident, $expected_type:literal, $type_:ty) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name;

        impl BuildSerializer for $struct_name {
            const EXPECTED_TYPE: &'static str = $expected_type;

            fn build(_schema: &PyDict, _config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
                Ok(Self {}.into())
            }
        }

        impl TypeSerializer for $struct_name {
            fn serde_serialize<S: serde::ser::Serializer>(
                &self,
                value: &PyAny,
                serializer: S,
                _include: Option<&PyAny>,
                _exclude: Option<&PyAny>,
                extra: &Extra,
            ) -> Result<S::Ok, S::Error> {
                match value.extract::<$type_>() {
                    Ok(v) => v.serialize(serializer),
                    Err(_) => {
                        extra.warnings.fallback_slow(Self::EXPECTED_TYPE, value);
                        fallback_serialize(value, serializer, extra.ob_type_lookup)
                    }
                }
            }
        }
    };
}

build_simple_serializer!(IntSerializer, "int", i64);
build_simple_serializer!(BoolSerializer, "bool", bool);
build_simple_serializer!(FloatSerializer, "float", f64);
