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

        // Note: we NEED to use the __dict__ here (and perform get_item calls rather than getattr)
        // because we don't want to fetch prebuilt validators from parent classes.
        // We don't downcast here because __dict__ on a class is a readonly mappingproxy,
        // so we can just leave it as is and do get_item checks.
        let class_dict = class.getattr(intern!(py, "__dict__"))?;

        let is_complete: bool = class_dict
            .get_item(intern!(py, "__pydantic_complete__"))
            .is_ok_and(|b| b.extract().unwrap_or(false));

        if !is_complete {
            return Err(PyValueError::new_err("Prebuilt validator not found."));
        }

        // Retrieve the prebuilt validator if available
        let prebuilt_validator = class_dict.get_item(intern!(py, "__pydantic_validator__"))?;
        let schema_validator: PyRef<SchemaValidator> = prebuilt_validator.extract()?;
        let combined_validator: Arc<CombinedValidator> = schema_validator.validator.clone();
        let name: String = class.getattr(intern!(py, "__name__"))?.extract()?;

        Ok(Self {
            validator: combined_validator,
            name,
        }
        .into())
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
