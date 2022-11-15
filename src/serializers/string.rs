use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use super::any::{fallback_serialize, ObTypeLookup};
use super::{py_err_se_err, BuildSerializer, CombinedSerializer, TypeSerializer};

#[derive(Debug, Clone)]
pub(super) struct StrSerializer;

impl BuildSerializer for StrSerializer {
    const EXPECTED_TYPE: &'static str = "str";

    fn build_combined(_schema: &PyDict, _config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        Ok(Self {}.into())
    }
}

impl TypeSerializer for StrSerializer {
    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        ob_type_lookup: &ObTypeLookup,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
    ) -> Result<S::Ok, S::Error> {
        match value.cast_as::<PyString>() {
            Ok(py_str) => serialize_py_str(py_str, serializer),
            Err(_) => fallback_serialize(value, serializer, ob_type_lookup),
        }
    }
}

pub(super) fn serialize_py_str<S: serde::ser::Serializer>(py_str: &PyString, serializer: S) -> Result<S::Ok, S::Error> {
    let s = py_str.to_str().map_err(py_err_se_err)?;
    serializer.serialize_str(s)
}
