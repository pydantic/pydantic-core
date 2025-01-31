use std::sync::Arc;

use pyo3::exceptions::PyValueError;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};

use crate::errors::ValResult;
use crate::input::Input;
use crate::tools::SchemaDict;

use super::ValidationState;
use super::{BuildValidator, CombinedValidator, DefinitionsBuilder, SchemaValidator, Validator};

#[derive(Debug)]
pub struct PrebuiltValidator {
    validator: Arc<CombinedValidator>,
    name: String,
}

impl BuildValidator for PrebuiltValidator {
    const EXPECTED_TYPE: &'static str = "prebuilt";

    fn build(
        schema: &Bound<'_, PyDict>,
        _config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let class: Bound<'_, PyType> = schema.get_as_req(intern!(py, "cls"))?;

        if class
            .getattr(intern!(py, "__pydantic_complete__"))
            .map_or(false, |pc| pc.extract::<bool>().unwrap_or(false))
        {
            if let Ok(prebuilt_validator) = class.getattr(intern!(py, "__pydantic_validator__")) {
                let schema_validator: PyRef<SchemaValidator> = prebuilt_validator.extract()?;
                let combined_validator: Arc<CombinedValidator> = schema_validator.validator.clone();
                let name = class.getattr(intern!(py, "__name__"))?.extract()?;

                return Ok(Self {
                    validator: combined_validator,
                    name,
                }
                .into());
            }
        }
        Err(PyValueError::new_err("Prebuilt validator not found."))
    }
}

impl_py_gc_traverse!(PrebuiltValidator { validator });

impl Validator for PrebuiltValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        self.validator.validate(py, input, state)
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}
