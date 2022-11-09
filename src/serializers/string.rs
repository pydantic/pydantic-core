use pyo3::prelude::*;
use pyo3::types::PyDict;

use super::{py_err_to_serde, BuildSerializer, CombinedSerializer, Serializer};

#[derive(Debug, Clone)]
pub struct StrSerializer;

impl BuildSerializer for StrSerializer {
    const EXPECTED_TYPE: &'static str = "str";

    fn build(_schema: &PyDict, _config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        Ok(Self {}.into())
    }
}

impl Serializer for StrSerializer {
    fn to_python(&self, py: Python, value: &PyAny, _format: Option<&str>) -> PyResult<PyObject> {
        Ok(value.into_py(py))
    }

    fn serde_serialize<S>(&self, value: &PyAny, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let s: &str = value.extract().map_err(py_err_to_serde)?;
        serializer.serialize_str(s)
    }
}
