use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use super::any::ObTypeLookup;
use super::{py_err_se_err, BuildSerializer, CombinedSerializer, TypeSerializer};

#[derive(Debug, Clone)]
pub struct StrSerializer;

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
        _ob_type_lookup: &ObTypeLookup,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
    ) -> Result<S::Ok, S::Error> {
        serialize_str(value, serializer)
    }
}

pub fn serialize_str<S: serde::ser::Serializer>(value: &PyAny, serializer: S) -> Result<S::Ok, S::Error> {
    let py_str: &PyString = value.cast_as().map_err(py_err_se_err)?;
    let s = py_str.to_str().map_err(py_err_se_err)?;
    serializer.serialize_str(s)
}
