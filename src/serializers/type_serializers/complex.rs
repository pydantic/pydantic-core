use std::borrow::Cow;

use pyo3::prelude::*;
use pyo3::types::{PyComplex, PyDict};

use serde::ser::SerializeMap;

use crate::definitions::DefinitionsBuilder;

use super::{infer_serialize, infer_to_python, BuildSerializer, CombinedSerializer, Extra, SerMode, TypeSerializer};

#[derive(Debug, Clone)]
pub struct ComplexSerializer {}

impl BuildSerializer for ComplexSerializer {
    const EXPECTED_TYPE: &'static str = "complex";
    fn build(
        _schema: &Bound<'_, PyDict>,
        _config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        Ok(Self {}.into())
    }
}

impl_py_gc_traverse!(ComplexSerializer {});

impl TypeSerializer for ComplexSerializer {
    fn to_python(
        &self,
        value: &Bound<'_, PyAny>,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let py = value.py();
        match value.downcast::<PyComplex>() {
            Ok(py_complex) => match extra.mode {
                SerMode::Json => {
                    let new_dict = PyDict::new_bound(py);
                    let _ = new_dict.set_item("real", py_complex.real());
                    let _ = new_dict.set_item("imag", py_complex.imag());
                    Ok(new_dict.into_py(py))
                }
                _ => Ok(value.into_py(py)),
            },
            Err(_) => {
                extra.warnings.on_fallback_py(self.get_name(), value, extra)?;
                infer_to_python(value, include, exclude, extra)
            }
        }
    }

    fn json_key<'a>(&self, key: &'a Bound<'_, PyAny>, extra: &Extra) -> PyResult<Cow<'a, str>> {
        self._invalid_as_json_key(key, extra, "complex")
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Bound<'_, PyAny>,
        serializer: S,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        match value.downcast::<PyComplex>() {
            Ok(py_complex) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry(&"real", &py_complex.real())?;
                map.serialize_entry(&"imag", &py_complex.imag())?;
                map.end()
            }
            Err(_) => {
                extra.warnings.on_fallback_ser::<S>(self.get_name(), value, extra)?;
                infer_serialize(value, serializer, include, exclude, extra)
            }
        }
    }

    fn get_name(&self) -> &str {
        "complex"
    }
}
