use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::build_context::BuildContext;
use crate::build_tools::{py_err, SchemaDict};
use crate::PydanticSerializationUnexpectedValue;

use super::any::{fallback_serialize, fallback_to_python};
use super::{py_err_se_err, BuildSerializer, CombinedSerializer, Extra, TypeSerializer};

#[derive(Debug, Clone)]
pub struct UnionSerializer {
    choices: Vec<CombinedSerializer>,
    name: String,
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

        match choices.len() {
            0 => py_err!("One or more union choices required"),
            1 => Ok(choices.into_iter().next().unwrap()),
            _ => {
                let descr = choices.iter().map(|v| v.get_name()).collect::<Vec<_>>().join(", ");
                Ok(Self {
                    choices,
                    name: format!("Union[{descr}]")
                }.into())
            }
        }
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
            .on_fallback_py(self.get_name(), value, error_on_fallback)?;
        fallback_to_python(value, include, exclude, extra)
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
        let py = value.py();
        for comb_serializer in &self.choices {
            match comb_serializer.to_python(value, include, exclude, extra, true) {
                Ok(v) => return fallback_serialize(v.as_ref(py), serializer, None, None, extra),
                Err(err) => match err.is_instance_of::<PydanticSerializationUnexpectedValue>(py) {
                    true => (),
                    false => return Err(py_err_se_err(err)),
                },
            }
        }

        extra
            .warnings
            .on_fallback_ser::<S>(self.get_name(), value, error_on_fallback)?;
        fallback_serialize(value, serializer, include, exclude, extra)
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}
