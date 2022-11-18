use std::str::FromStr;

use pyo3::exceptions::PyRuntimeError;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use serde::ser::Error;

use crate::build_tools::{function_name, kwargs, py_error_type, SchemaDict};
use crate::serializers::any::{fallback_serialize, fallback_to_python_json, ob_type_to_python_json};

use super::any::{fallback_serialize_known, ObType};
use super::shared::{BuildSerializer, CombinedSerializer, Extra, SerFormat, TypeSerializer};

#[derive(Debug, Clone)]
pub struct FunctionSerializer {
    func: PyObject,
    function_name: String,
    return_ob_type: Option<ObType>,
}

impl BuildSerializer for FunctionSerializer {
    // this value is never used, it's just here to satisfy the trait
    const EXPECTED_TYPE: &'static str = "";

    fn build(schema: &PyDict, _config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let function = schema.get_as_req::<&PyAny>(intern!(py, "function"))?;
        let function_name = function_name(function)?;
        Ok(Self {
            func: function.into_py(py),
            function_name,
            return_ob_type: schema
                .get_as::<&str>(intern!(py, "type"))?
                .map(|t| ObType::from_str(t).unwrap()),
        }
        .into())
    }
}
impl FunctionSerializer {
    fn call(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        format: &SerFormat,
    ) -> PyResult<PyObject> {
        let py = value.py();
        let kwargs = kwargs!(py, format: format.to_object(py), include: include, exclude: exclude);
        self.func.call(py, (value,), kwargs)
    }
}

impl TypeSerializer for FunctionSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let py = value.py();
        let v = self
            .call(value, include, exclude, extra.format)
            .map_err(|e| py_error_type!(PyRuntimeError; "Error calling `{}`: {}", self.function_name, e))?;

        match extra.format {
            SerFormat::Json => {
                if let Some(ref ob_type) = self.return_ob_type {
                    ob_type_to_python_json(ob_type, v.as_ref(py), extra.ob_type_lookup)
                } else {
                    fallback_to_python_json(v.as_ref(py), extra.ob_type_lookup)
                }
            }
            _ => Ok(v),
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        let py = value.py();
        let return_value = self
            .call(value, include, exclude, extra.format)
            .map_err(|e| Error::custom(format!("Error calling `{}`: {}", self.function_name, e)))?;

        if let Some(ref ob_type) = self.return_ob_type {
            fallback_serialize_known(ob_type, return_value.as_ref(py), serializer, extra.ob_type_lookup)
        } else {
            fallback_serialize(return_value.as_ref(py), serializer, extra.ob_type_lookup)
        }
    }
}
