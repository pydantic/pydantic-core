use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::errors::ValResult;
use crate::input::Input;

use super::{BuildValidator, CombinedValidator, DefinitionsBuilder, ValidationState, Validator};

#[derive(Debug)]
pub struct ComplexValidator {}

impl BuildValidator for ComplexValidator {
    const EXPECTED_TYPE: &'static str = "complex";
    fn build(
        _schema: &Bound<'_, PyDict>,
        _config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        Ok(Self {}.into())
    }
}

impl_py_gc_traverse!(ComplexValidator {});

impl Validator for ComplexValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        let res = input.validate_complex()?.unpack(state);
        Ok(res.into_py(py))
    }

    fn get_name(&self) -> &str {
        "complex"
    }
}
