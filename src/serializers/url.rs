use std::borrow::Cow;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::url::{PyMultiHostUrl, PyUrl};

use super::any::{fallback_serialize, fallback_to_python_json, json_key};
use super::shared::{BuildSerializer, CombinedSerializer, Extra, SerMode, TypeSerializer};

macro_rules! build_serializer {
    ($struct_name:ident, $expected_type:literal, $extract:ty) => {
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
                match extra.mode {
                    SerMode::Json => match value.extract::<$extract>() {
                        Ok(py_url) => Ok(py_url.__str__().into_py(py)),
                        Err(_) => {
                            extra.warnings.fallback_slow(Self::EXPECTED_TYPE, value);
                            fallback_to_python_json(value, extra)
                        }
                    },
                    _ => Ok(value.into_py(py)),
                }
            }

            fn json_key<'py>(&self, key: &'py PyAny, extra: &Extra) -> PyResult<Cow<'py, str>> {
                match key.extract::<$extract>() {
                    Ok(py_url) => Ok(Cow::Owned(py_url.__str__().to_string())),
                    Err(_) => {
                        extra.warnings.fallback_slow(Self::EXPECTED_TYPE, key);
                        json_key(key, extra)
                    }
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
                match value.extract::<$extract>() {
                    Ok(py_url) => serializer.serialize_str(&py_url.__str__()),
                    Err(_) => {
                        extra.warnings.fallback_slow(Self::EXPECTED_TYPE, value);
                        fallback_serialize(value, serializer, extra)
                    }
                }
            }
        }
    };
}
build_serializer!(UrlSerializer, "url", PyUrl);
build_serializer!(MultiHostUrlSerializer, "multi-host-url", PyMultiHostUrl);
