use pyo3::prelude::*;

use pyo3::types::PyDict;

use crate::errors::ValResult;
use crate::input::Input;
use crate::tools::SchemaDict;

use crate::recursion_guard::RecursionGuard;

use super::chain::ChainValidator;
use super::{BuildValidator, CombinedValidator, Definitions, DefinitionsBuilder, Extra, Validator};

#[derive(Debug, Clone)]
pub struct LengthConstraint {
    min_length: Option<usize>,
    max_length: Option<usize>,
}

impl LengthConstraint {
    pub fn maybe_wrap(schema: &PyDict, validator: CombinedValidator) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let min_length: Option<usize> = schema.get_as(pyo3::intern!(py, "min_length"))?;
        let max_length: Option<usize> = schema.get_as(pyo3::intern!(py, "max_length"))?;
        match min_length.or(max_length) {
            Some(_) => Ok(ChainValidator::new(
                vec![validator, Self { min_length, max_length }.into()],
                "length_constraint".to_string(),
            )
            .into()),
            None => Ok(validator),
        }
    }
}

impl BuildValidator for LengthConstraint {
    const EXPECTED_TYPE: &'static str = "bool";

    fn build(
        schema: &PyDict,
        _config: Option<&PyDict>,
        _definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        Ok(CombinedValidator::LengthConstraint(Self {
            min_length: schema.get_as(pyo3::intern!(py, "min_length"))?,
            max_length: schema.get_as(pyo3::intern!(py, "max_length"))?,
        }))
    }
}

impl_py_gc_traverse!(LengthConstraint {});

impl Validator for LengthConstraint {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        _extra: &Extra,
        _definitions: &'data Definitions<CombinedValidator>,
        _recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let mut len: Option<usize> = None;
        if let Some(min_length) = self.min_length {
            let len = match len {
                Some(l) => l,
                None => {
                    let l = input.len(py)?;
                    len = Some(l);
                    l
                }
            };
            if len < min_length {
                return Err(crate::errors::ValError::new(
                    crate::errors::ErrorType::TooShort {
                        min_length,
                        actual_length: len,
                    },
                    input,
                ));
            }
        }
        if let Some(max_length) = self.max_length {
            let len = match len {
                Some(l) => l,
                None => input.to_object(py).as_ref(py).len()?,
            };
            if len > max_length {
                return Err(crate::errors::ValError::new(
                    crate::errors::ErrorType::TooLong {
                        max_length,
                        actual_length: len,
                    },
                    input,
                ));
            }
        }
        Ok(input.to_object(py))
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
