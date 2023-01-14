use std::borrow::Cow;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_context::BuildContext;
use crate::url::{PyMultiHostUrl, PyUrl};

use super::any::{fallback_json_key, fallback_serialize, fallback_to_python};
use super::{BuildSerializer, CombinedSerializer, Extra, SerMode, TypeSerializer};

macro_rules! build_serializer {
    ($struct_name:ident, $expected_type:literal, $extract:ty) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name;

        impl BuildSerializer for $struct_name {
            const EXPECTED_TYPE: &'static str = $expected_type;

            fn build(
                _schema: &PyDict,
                _config: Option<&PyDict>,
                _build_context: &mut BuildContext<CombinedSerializer>,
            ) -> PyResult<CombinedSerializer> {
                Ok(Self {}.into())
            }
        }

        impl TypeSerializer for $struct_name {
            fn to_python(
                &self,
                value: &PyAny,
                include: Option<&PyAny>,
                exclude: Option<&PyAny>,
                extra: &Extra,
                error_on_fallback: bool,
            ) -> PyResult<PyObject> {
                let py = value.py();
                match value.extract::<$extract>() {
                    Ok(py_url) => match extra.mode {
                        SerMode::Json => Ok(py_url.__str__().into_py(py)),
                        _ => Ok(value.into_py(py)),
                    },
                    Err(_) => {
                        extra
                            .warnings
                            .on_fallback_py(Self::EXPECTED_TYPE, value, error_on_fallback)?;
                        fallback_to_python(value, include, exclude, extra)
                    }
                }
            }

            fn json_key<'py>(
                &self,
                key: &'py PyAny,
                extra: &Extra,
                error_on_fallback: bool,
            ) -> PyResult<Cow<'py, str>> {
                match key.extract::<$extract>() {
                    Ok(py_url) => Ok(Cow::Owned(py_url.__str__().to_string())),
                    Err(_) => {
                        extra
                            .warnings
                            .on_fallback_py(Self::EXPECTED_TYPE, key, error_on_fallback)?;
                        fallback_json_key(key, extra)
                    }
                }
            }

            fn serde_serialize<S: serde::ser::Serializer>(
                &self,
                value: &PyAny,
                serializer: S,
                include: Option<&PyAny>,
                exclude: Option<&PyAny>,
                extra: &Extra,
                error_on_fallback: bool,
            ) -> Result<S::Ok, S::Error> {
                match value.extract::<$extract>() {
                    Ok(py_url) => serializer.serialize_str(&py_url.__str__()),
                    Err(_) => {
                        extra
                            .warnings
                            .on_fallback_ser::<S>(Self::EXPECTED_TYPE, value, error_on_fallback)?;
                        fallback_serialize(value, serializer, include, exclude, extra)
                    }
                }
            }
        }
    };
}
build_serializer!(UrlSerializer, "url", PyUrl);
build_serializer!(MultiHostUrlSerializer, "multi-host-url", PyMultiHostUrl);
