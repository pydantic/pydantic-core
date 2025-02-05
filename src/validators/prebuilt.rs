use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};

use crate::errors::ValResult;
use crate::input::Input;
use crate::tools::SchemaDict;

use super::ValidationState;
use super::{CombinedValidator, SchemaValidator, Validator};

#[derive(Debug)]
pub struct PrebuiltValidator {
    schema_validator: Py<SchemaValidator>,
}

impl PrebuiltValidator {
    pub fn try_get_from_schema(type_: &str, schema: &Bound<'_, PyDict>) -> PyResult<Option<CombinedValidator>> {
        let py = schema.py();

        // we can only use prebuilt validators from models, typed dicts, and dataclasses
        // however, we don't want to use a prebuilt validator for dataclasses if we have a generic_origin
        // because __pydantic_validator__ is cached on the unparametrized dataclass
        if !matches!(type_, "model" | "typed-dict")
            || matches!(type_, "dataclass") && schema.contains(intern!(py, "generic_origin"))?
        {
            return Ok(None);
        }

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
            return Ok(None);
        }

        // Retrieve the prebuilt validator if available
        let prebuilt_validator = class_dict.get_item(intern!(py, "__pydantic_validator__"))?;
        let schema_validator = prebuilt_validator.extract::<Py<SchemaValidator>>()?;

        Ok(Some(Self { schema_validator }.into()))
    }
}

impl_py_gc_traverse!(PrebuiltValidator { schema_validator });

impl Validator for PrebuiltValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        self.schema_validator.get().validator.validate(py, input, state)
    }

    fn get_name(&self) -> &str {
        self.schema_validator.get().validator.get_name()
    }
}
