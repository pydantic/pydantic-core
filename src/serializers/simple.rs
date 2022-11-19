use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::Serialize;

use super::any::{fallback_serialize, fallback_to_python_json, ObType};
use super::shared::{BuildSerializer, CombinedSerializer, Extra, SerFormat, TypeSerializer};

#[derive(Debug, Clone)]
pub struct NoneSerializer;

impl BuildSerializer for NoneSerializer {
    const EXPECTED_TYPE: &'static str = "none";

    fn build(_schema: &PyDict, _config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        Ok(Self {}.into())
    }
}

impl TypeSerializer for NoneSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let py = value.py();
        match extra.format {
            SerFormat::Json => match extra.ob_type_lookup.is_type(value, ObType::None) {
                true => Ok(py.None().into_py(py)),
                false => {
                    extra.warnings.fallback_slow(Self::EXPECTED_TYPE, value);
                    fallback_to_python_json(value, extra.ob_type_lookup)
                }
            },
            _ => Ok(value.into_py(py)),
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        match extra.ob_type_lookup.is_type(value, ObType::None) {
            true => serializer.serialize_none(),
            false => {
                extra.warnings.fallback_slow(Self::EXPECTED_TYPE, value);
                fallback_serialize(value, serializer, extra.ob_type_lookup)
            }
        }
    }
}

macro_rules! build_simple_serializer {
    ($struct_name:ident, $expected_type:literal, $rust_type:ty, $ob_type:expr) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name;

        impl BuildSerializer for $struct_name {
            const EXPECTED_TYPE: &'static str = $expected_type;

            fn build(_schema: &PyDict, _config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
                Ok(Self {}.into())
            }
        }

        impl TypeSerializer for $struct_name {
            fn to_python(
                &self,
                value: &PyAny,
                _include: Option<&PyAny>,
                _exclude: Option<&PyAny>,
                extra: &Extra,
            ) -> PyResult<PyObject> {
                let py = value.py();
                match extra.format {
                    SerFormat::Json => match extra.ob_type_lookup.is_type(value, $ob_type) {
                        true => Ok(value.into_py(py)),
                        false => {
                            extra.warnings.fallback_slow(Self::EXPECTED_TYPE, value);
                            fallback_to_python_json(value, extra.ob_type_lookup)
                        }
                    },
                    _ => Ok(value.into_py(py)),
                }
            }

            fn serde_serialize<S: serde::ser::Serializer>(
                &self,
                value: &PyAny,
                serializer: S,
                _include: Option<&PyAny>,
                _exclude: Option<&PyAny>,
                extra: &Extra,
            ) -> Result<S::Ok, S::Error> {
                match value.extract::<$rust_type>() {
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

build_simple_serializer!(IntSerializer, "int", i64, ObType::Int);
build_simple_serializer!(BoolSerializer, "bool", bool, ObType::Bool);
build_simple_serializer!(FloatSerializer, "float", f64, ObType::Float);
