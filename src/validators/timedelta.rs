use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::is_strict;
use crate::errors::ValResult;
use crate::input::Input;
use crate::recursion_guard::RecursionGuard;

use super::datetime::extract_microseconds_precision;
use super::{BuildValidator, CombinedValidator, Definitions, DefinitionsBuilder, Extra, Validator};

#[derive(Debug, Clone)]
pub struct TimeDeltaValidator {
    strict: bool,
    microseconds_precision: speedate::MicrosecondsPrecisionOverflowBehavior,
}

impl BuildValidator for TimeDeltaValidator {
    const EXPECTED_TYPE: &'static str = "timedelta";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        _definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        Ok(Self {
            strict: is_strict(schema, config)?,
            microseconds_precision: extract_microseconds_precision(schema, config)?,
        }
        .into())
    }
}

impl_py_gc_traverse!(TimeDeltaValidator {});

impl Validator for TimeDeltaValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        _definitions: &'data Definitions<CombinedValidator>,
        _recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let timedelta = input.validate_timedelta(extra.strict.unwrap_or(self.strict), self.microseconds_precision)?;
        let py_timedelta = timedelta.try_into_py(py)?;
        Ok(py_timedelta.into())
    }

    fn different_strict_behavior(
        &self,
        _definitions: Option<&DefinitionsBuilder<CombinedValidator>>,
        ultra_strict: bool,
    ) -> bool {
        !ultra_strict
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }

    fn complete(&mut self, _definitions: &DefinitionsBuilder<CombinedValidator>) -> PyResult<()> {
        Ok(())
    }
}
