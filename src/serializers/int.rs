use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::Serialize;

use super::any::{fallback_serialize, ObTypeLookup};
use super::{BuildSerializer, CombinedSerializer, TypeSerializer};

#[derive(Debug, Clone)]
pub(super) struct IntSerializer;

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
        ob_type_lookup: &ObTypeLookup,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
    ) -> Result<S::Ok, S::Error> {
        match value.extract::<i64>() {
            Ok(v) => v.serialize(serializer),
            Err(_) => fallback_serialize(value, serializer, ob_type_lookup),
        }
    }
}
