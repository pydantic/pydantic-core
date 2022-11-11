use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use super::any::ObTypeLookup;
use super::{py_err_to_serde, BuildSerializer, CombinedSerializer, TypeSerializer};

#[derive(Debug, Clone)]
pub struct StrSerializer;

impl BuildSerializer for StrSerializer {
    const EXPECTED_TYPE: &'static str = "str";

    fn build(_schema: &PyDict, _config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        Ok(Self {}.into())
    }
}

impl TypeSerializer for StrSerializer {
    fn to_python(&self, py: Python, value: &PyAny, _format: Option<&str>) -> PyResult<PyObject> {
        Ok(value.into_py(py))
    }

    fn serde_serialize<S>(
        &self,
        value: &PyAny,
        serializer: S,
        _ob_type_lookup: &ObTypeLookup,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let py_str: &PyString = value.cast_as().map_err(py_err_to_serde)?;
        let s = py_str.to_str().map_err(py_err_to_serde)?;
        serializer.serialize_str(s)
    }
}
