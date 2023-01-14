use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::build_context::BuildContext;
use crate::build_tools::SchemaDict;
use crate::PydanticSerializationUnexpectedValue;

use super::any::{fallback_serialize, fallback_to_python};
use super::{BuildSerializer, CombinedSerializer, Extra, TypeSerializer};

#[derive(Debug, Clone)]
pub struct UnionSerializer {
    choices: Vec<CombinedSerializer>,
}

impl BuildSerializer for UnionSerializer {
    const EXPECTED_TYPE: &'static str = "union";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let choices: Vec<CombinedSerializer> = schema
            .get_as_req::<&PyList>(intern!(py, "choices"))?
            .iter()
            .map(|choice| CombinedSerializer::build(choice.cast_as()?, config, build_context))
            .collect::<PyResult<Vec<CombinedSerializer>>>()?;

        Ok(Self { choices }.into())
    }
}

impl TypeSerializer for UnionSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
        error_on_fallback: bool,
    ) -> PyResult<PyObject> {
        // try the serializers in with error_on fallback=true
        for comb_serializer in &self.choices {
            match comb_serializer.to_python(value, include, exclude, extra, true) {
                Ok(v) => return Ok(v),
                Err(err) => match err.is_instance_of::<PydanticSerializationUnexpectedValue>(value.py()) {
                    true => (),
                    false => return Err(err),
                },
            }
        }
        extra
            .warnings
            .on_fallback_py(Self::EXPECTED_TYPE, value, error_on_fallback)?;
        fallback_to_python(value, include, exclude, extra, error_on_fallback)
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
        // try the serializers in with error_on fallback=true
        // for comb_serializer in &self.choices {
        //     match comb_serializer.serde_serialize(value, serializer, include, exclude, extra, true) {
        //         Ok(v) => return Ok(v),
        //         Err(_e) => (),
        //     }
        // }

        extra
            .warnings
            .on_fallback_ser::<S>(Self::EXPECTED_TYPE, value, error_on_fallback)?;
        fallback_serialize(value, serializer, include, exclude, extra, error_on_fallback)
    }
}
