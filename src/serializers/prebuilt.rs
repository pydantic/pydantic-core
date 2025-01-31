use std::borrow::Cow;
use std::sync::Arc;

use pyo3::exceptions::PyValueError;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyType};

use crate::definitions::DefinitionsBuilder;
use crate::tools::SchemaDict;
use crate::SchemaSerializer;

use super::extra::Extra;
use super::shared::{BuildSerializer, CombinedSerializer, TypeSerializer};

#[derive(Debug)]
pub struct PrebuiltSerializer {
    serializer: Arc<CombinedSerializer>,
}

impl BuildSerializer for PrebuiltSerializer {
    const EXPECTED_TYPE: &'static str = "prebuilt";

    fn build(
        schema: &Bound<'_, PyDict>,
        _config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let class: Bound<'_, PyType> = schema.get_as_req(intern!(py, "cls"))?;

        // note: we NEED to use the __dict__ here (and perform get_item calls rather than getattr)
        // because we don't want to fetch prebuilt serializers from parent classes
        let class_dict: Bound<'_, PyDict> = class.getattr(intern!(py, "__dict__"))?.extract()?;

        // Ensure the class has completed its Pydantic setup
        let is_complete: bool = class_dict
            .get_as_req::<Bound<'_, PyBool>>(intern!(py, "__pydantic_complete__"))
            .is_ok_and(|b| b.extract().unwrap_or(false));

        if !is_complete {
            return Err(PyValueError::new_err("Prebuilt serializer not found."));
        }

        // Retrieve the prebuilt validator if available
        let prebuilt_serializer: Bound<'_, PyAny> = class_dict.get_as_req(intern!(py, "__pydantic_serializer__"))?;
        let schema_serializer: PyRef<SchemaSerializer> = prebuilt_serializer.extract()?;
        let combined_serializer: Arc<CombinedSerializer> = schema_serializer.serializer.clone();

        Ok(Self {
            serializer: combined_serializer,
        }
        .into())
    }
}

impl_py_gc_traverse!(PrebuiltSerializer { serializer });

impl TypeSerializer for PrebuiltSerializer {
    fn to_python(
        &self,
        value: &Bound<'_, PyAny>,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        self.serializer.to_python(value, include, exclude, extra)
    }

    fn json_key<'a>(&self, key: &'a Bound<'_, PyAny>, extra: &Extra) -> PyResult<Cow<'a, str>> {
        self.serializer.json_key(key, extra)
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Bound<'_, PyAny>,
        serializer: S,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        self.serializer
            .serde_serialize(value, serializer, include, exclude, extra)
    }

    fn get_name(&self) -> &str {
        self.serializer.get_name()
    }

    fn retry_with_lax_check(&self) -> bool {
        self.serializer.retry_with_lax_check()
    }
}
