use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::Serialize;

use super::any::ObTypeLookup;
use super::{py_err_se_err, BuildSerializer, CombinedSerializer, TypeSerializer};

#[derive(Debug, Clone)]
pub struct IntSerializer;

impl BuildSerializer for IntSerializer {
    const EXPECTED_TYPE: &'static str = "int";

    fn build_combined(_schema: &PyDict, _config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        Ok(Self {}.into())
    }
}

impl TypeSerializer for IntSerializer {
    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        _ob_type_lookup: &ObTypeLookup,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
    ) -> Result<S::Ok, S::Error> {
        serialize_int(value, serializer)
    }
}

pub fn serialize_int<S: serde::ser::Serializer>(value: &PyAny, serializer: S) -> Result<S::Ok, S::Error> {
    match value.extract::<i64>() {
        Ok(v) => v.serialize(serializer),
        Err(e) => Err(py_err_se_err(e)),
    }
}
