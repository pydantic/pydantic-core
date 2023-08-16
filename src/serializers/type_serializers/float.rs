use pyo3::types::PyDict;
use pyo3::{intern, prelude::*};

use std::borrow::Cow;

use serde::Serializer;

use crate::definitions::DefinitionsBuilder;
use crate::tools::SchemaDict;

use super::simple::to_str_json_key;
use super::{
    infer_json_key, infer_serialize, infer_to_python, BuildSerializer, CombinedSerializer, Extra, IsType, ObType,
    SerMode, TypeSerializer,
};

#[derive(Debug, Clone)]
pub struct FloatSerializer {
    allow_inf_nan: bool,
}

impl BuildSerializer for FloatSerializer {
    const EXPECTED_TYPE: &'static str = "float";

    fn build(
        schema: &PyDict,
        _config: Option<&PyDict>,
        _definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let allow_inf_nan = schema
            .get_as::<bool>(intern!(schema.py(), "allow_inf_nan"))?
            .unwrap_or(false);
        Ok(Self { allow_inf_nan }.into())
    }
}

impl_py_gc_traverse!(FloatSerializer {});

impl TypeSerializer for FloatSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let py = value.py();
        match extra.ob_type_lookup.is_type(value, ObType::Float) {
            IsType::Exact => Ok(value.into_py(py)),
            IsType::Subclass => match extra.mode {
                SerMode::Json => {
                    let rust_value = value.extract::<f64>()?;
                    Ok(rust_value.to_object(py))
                }
                _ => infer_to_python(value, include, exclude, extra),
            },
            IsType::False => {
                extra.warnings.on_fallback_py(self.get_name(), value, extra)?;
                infer_to_python(value, include, exclude, extra)
            }
        }
    }

    fn json_key<'py>(&self, key: &'py PyAny, extra: &Extra) -> PyResult<Cow<'py, str>> {
        match extra.ob_type_lookup.is_type(key, ObType::Float) {
            IsType::Exact | IsType::Subclass => to_str_json_key(key),
            IsType::False => {
                extra.warnings.on_fallback_py(self.get_name(), key, extra)?;
                infer_json_key(key, extra)
            }
        }
    }

    fn serde_serialize<S: Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        match value.extract::<f64>() {
            Ok(v) => {
                if (v.is_nan() || v.is_infinite()) && !self.allow_inf_nan {
                    return serializer.serialize_none();
                }
                serializer.serialize_f64(v)
            }
            Err(_) => {
                extra.warnings.on_fallback_ser::<S>(self.get_name(), value, extra)?;
                infer_serialize(value, serializer, include, exclude, extra)
            }
        }
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}
