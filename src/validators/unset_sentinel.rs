use core::fmt::Debug;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::common::unset_sentinel::get_unset_sentinel_object;
use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::Input;

use super::{BuildValidator, CombinedValidator, DefinitionsBuilder, ValidationState, Validator};

#[derive(Debug, Clone)]
pub struct UnsetSentinelValidator {}

impl BuildValidator for UnsetSentinelValidator {
    const EXPECTED_TYPE: &'static str = "unset-sentinel";

    fn build(
        _schema: &Bound<'_, PyDict>,
        _config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        Ok(CombinedValidator::UnsetSentinel(Self {}))
    }
}

impl_py_gc_traverse!(UnsetSentinelValidator {});

impl Validator for UnsetSentinelValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        _state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        let unset_obj = get_unset_sentinel_object(py);

        match input.as_python() {
            Some(v) if v.is(unset_obj) => Ok(v.to_owned().into()),
            _ => Err(ValError::new(ErrorType::UnsetSentinelError { context: None }, input)),
        }
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}
