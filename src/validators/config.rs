use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::config::CoreConfig;
use crate::errors::ValResult;
use crate::input::Input;

use super::{CombinedValidator, ValidationState, Validator};

/// A validator that sets the current configuration.
#[derive(Debug, Clone)]
pub struct ConfigValidator {
    config: CoreConfig,
    inner: Arc<CombinedValidator>,
}

impl ConfigValidator {
    pub fn try_new(config: Bound<'_, PyDict>, inner: Arc<CombinedValidator>) -> PyResult<Self> {
        Ok(Self {
            config: config.try_into()?,
            inner,
        })
    }
}

impl_py_gc_traverse!(ConfigValidator {});

impl Validator for ConfigValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        let mut state =
            ValidationState::new_with_config(state.extra().clone(), state.recursion_guard, self.config.clone());
        self.inner.validate(py, input, &mut state)
    }

    fn get_name(&self) -> &str {
        "config"
    }
}
