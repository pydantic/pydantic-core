use num_bigint::BigInt;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::is_strict;
use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::Exactness;
use crate::input::{Input, Int};
use crate::recursion_guard::RecursionGuard;
use crate::tools::SchemaDict;

use super::Validation;
use super::{BuildValidator, CombinedValidator, Definitions, DefinitionsBuilder, Extra, Validator};

#[derive(Debug, Clone)]
pub struct IntValidator {
    strict: bool,
}

impl BuildValidator for IntValidator {
    const EXPECTED_TYPE: &'static str = "int";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        _definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let use_constrained = schema.get_item(intern!(py, "multiple_of")).is_some()
            || schema.get_item(intern!(py, "le")).is_some()
            || schema.get_item(intern!(py, "lt")).is_some()
            || schema.get_item(intern!(py, "ge")).is_some()
            || schema.get_item(intern!(py, "gt")).is_some();
        if use_constrained {
            ConstrainedIntValidator::build(schema, config)
        } else {
            Ok(Self {
                strict: is_strict(schema, config)?,
            }
            .into())
        }
    }
}

impl_py_gc_traverse!(IntValidator {});

impl Validator for IntValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        _definitions: &'data Definitions<CombinedValidator>,
        _recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, Validation<PyObject>> {
        let strict = extra.strict.unwrap_or(self.strict);
        let Some((parse_result, exactness)) = input.validate_int() else { return Err(ValError::new(ErrorType::IntType, input)) };
        if strict && matches!(exactness, Exactness::Lax) {
            return Err(ValError::new(ErrorType::IntType, input));
        }
        Ok(Validation::new(parse_result?.into_py(py), exactness))
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

#[derive(Debug, Clone)]
pub struct ConstrainedIntValidator {
    strict: bool,
    multiple_of: Option<Int>,
    le: Option<Int>,
    lt: Option<Int>,
    ge: Option<Int>,
    gt: Option<Int>,
}

impl_py_gc_traverse!(ConstrainedIntValidator {});

impl Validator for ConstrainedIntValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        _definitions: &'data Definitions<CombinedValidator>,
        _recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, Validation<PyObject>> {
        let strict = extra.strict.unwrap_or(self.strict);
        let Some((parse_result, exactness)) = input.validate_int() else { return Err(ValError::new(ErrorType::IntType, input)) };
        if strict && matches!(exactness, Exactness::Lax) {
            return Err(ValError::new(ErrorType::IntType, input));
        }
        let either_int = parse_result?;
        let int_value = either_int.as_int()?;

        if let Some(ref multiple_of) = self.multiple_of {
            if &int_value % multiple_of != Int::Big(BigInt::from(0)) {
                return Err(ValError::new(
                    ErrorType::MultipleOf {
                        multiple_of: multiple_of.clone().into(),
                    },
                    input,
                ));
            }
        }
        if let Some(ref le) = self.le {
            if &int_value > le {
                return Err(ValError::new(ErrorType::LessThanEqual { le: le.clone().into() }, input));
            }
        }
        if let Some(ref lt) = self.lt {
            if &int_value >= lt {
                return Err(ValError::new(ErrorType::LessThan { lt: lt.clone().into() }, input));
            }
        }
        if let Some(ref ge) = self.ge {
            if &int_value < ge {
                return Err(ValError::new(
                    ErrorType::GreaterThanEqual { ge: ge.clone().into() },
                    input,
                ));
            }
        }
        if let Some(ref gt) = self.gt {
            if &int_value <= gt {
                return Err(ValError::new(ErrorType::GreaterThan { gt: gt.clone().into() }, input));
            }
        }
        Ok(Validation::new(either_int.into_py(py), exactness))
    }

    fn different_strict_behavior(
        &self,
        _definitions: Option<&DefinitionsBuilder<CombinedValidator>>,
        ultra_strict: bool,
    ) -> bool {
        !ultra_strict
    }

    fn get_name(&self) -> &str {
        "constrained-int"
    }

    fn complete(&mut self, _definitions: &DefinitionsBuilder<CombinedValidator>) -> PyResult<()> {
        Ok(())
    }
}

impl ConstrainedIntValidator {
    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<CombinedValidator> {
        let py = schema.py();
        Ok(Self {
            strict: is_strict(schema, config)?,
            multiple_of: schema.get_as(intern!(py, "multiple_of"))?,
            le: schema.get_as(intern!(py, "le"))?,
            lt: schema.get_as(intern!(py, "lt"))?,
            ge: schema.get_as(intern!(py, "ge"))?,
            gt: schema.get_as(intern!(py, "gt"))?,
        }
        .into())
    }
}
