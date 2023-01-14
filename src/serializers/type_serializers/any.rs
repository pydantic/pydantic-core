use pyo3::prelude::*;
use pyo3::types::PyDict;

use serde::ser::Serializer;

use crate::build_context::BuildContext;

use super::{infer_serialize, BuildSerializer, CombinedSerializer, Extra, TypeSerializer};

#[derive(Debug, Clone)]
pub struct AnySerializer;

impl BuildSerializer for AnySerializer {
    const EXPECTED_TYPE: &'static str = "any";

    fn build(
        _schema: &PyDict,
        _config: Option<&PyDict>,
        _build_context: &mut BuildContext<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        Ok(Self {}.into())
    }
}

impl TypeSerializer for AnySerializer {
    fn serde_serialize<S: Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        infer_serialize(value, serializer, include, exclude, extra)
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}
