use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_context::BuildContext;
use crate::build_tools::SchemaDict;

use super::{object_to_dict, py_err_se_err, BuildSerializer, CombinedSerializer, Extra, TypeSerializer};

#[derive(Debug, Clone)]
pub struct NewClassSerializer {
    serializer: Box<CombinedSerializer>,
}

impl BuildSerializer for NewClassSerializer {
    const EXPECTED_TYPE: &'static str = "new-class";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let sub_schema: &PyDict = schema.get_as_req(intern!(py, "schema"))?;
        let serializer = Box::new(CombinedSerializer::build(sub_schema, config, build_context)?);

        Ok(Self { serializer }.into())
    }
}

impl TypeSerializer for NewClassSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
        error_on_fallback: bool,
    ) -> PyResult<PyObject> {
        let dict = object_to_dict(value, true, extra)?;
        self.serializer
            .to_python(dict, include, exclude, extra, error_on_fallback)
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
        let dict = object_to_dict(value, true, extra).map_err(py_err_se_err)?;
        self.serializer
            .serde_serialize(dict, serializer, include, exclude, extra, error_on_fallback)
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}
