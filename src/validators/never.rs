use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::errors::{ErrorTypeDefaults, ValError, ValResult};
use crate::input::Input;
use crate::PydanticUndefinedType;

use super::{BuildValidator, CombinedValidator, DefinitionsBuilder, LocItem, ValidationState, Validator};

#[derive(Debug)]
pub struct NeverValidator {
    undefined: PyObject,
}

impl BuildValidator for NeverValidator {
    const EXPECTED_TYPE: &'static str = "never";

    fn build(
        schema: &Bound<'_, PyDict>,
        _config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        Ok(Self {
            undefined: PydanticUndefinedType::new(py).into_any(),
        }
        .into())
    }
}

impl_py_gc_traverse!(NeverValidator {});

impl Validator for NeverValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        _state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        let obj = input.to_object(py)?;
        if obj.is(&self.undefined) {
            Ok(obj.into())
        } else {
            Err(ValError::new(ErrorTypeDefaults::Never, input))
        }
    }

    fn default_value<'py>(
        &self,
        _py: Python<'py>,
        _outer_loc: Option<impl Into<LocItem>>,
        _state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<Option<PyObject>> {
        Ok(Some(self.undefined.clone()))
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}
