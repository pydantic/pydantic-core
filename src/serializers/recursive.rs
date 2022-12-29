use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_context::BuildContext;
use crate::build_tools::SchemaDict;

use super::{BuildSerializer, CombinedSerializer, Extra, TypeSerializer};

#[derive(Debug, Clone)]
pub struct RecursiveRefSerializer {
    serializer_id: usize,
}

impl RecursiveRefSerializer {
    pub fn from_id(serializer_id: usize) -> CombinedSerializer {
        Self { serializer_id }.into()
    }
}

impl BuildSerializer for RecursiveRefSerializer {
    const EXPECTED_TYPE: &'static str = "recursive-ref";

    fn build(
        schema: &PyDict,
        _config: Option<&PyDict>,
        build_context: &mut BuildContext<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let name: String = schema.get_as_req(intern!(schema.py(), "schema_ref"))?;
        let (serializer_id, _) = build_context.find_slot_id_answer(&name)?;
        Ok(Self { serializer_id }.into())
    }
}

impl TypeSerializer for RecursiveRefSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let comb_serializer = unsafe { extra.slots.get_unchecked(self.serializer_id) };
        comb_serializer.to_python(value, include, exclude, extra)
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        let comb_serializer = unsafe { extra.slots.get_unchecked(self.serializer_id) };
        comb_serializer.serde_serialize(value, serializer, include, exclude, extra)
    }
}
