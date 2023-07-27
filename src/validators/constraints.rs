use pyo3::pyclass::CompareOp;
use pyo3::{intern, prelude::*};

use pyo3::types::PyDict;

use crate::SchemaError;
use crate::errors::ValResult;
use crate::input::Input;
use crate::py_gc::PyGcTraverse;
use crate::tools::SchemaDict;

use crate::recursion_guard::RecursionGuard;

use super::{BuildValidator, CombinedValidator, Definitions, DefinitionsBuilder, Extra, Validator};

#[derive(Debug, Clone)]
pub enum Constraint {
    // TODO: optimize these with an EitherInt like thing
    MinLength(usize),
    MaxLength(usize),
    GreaterThan(PyObject),
    GreaterThanOrEqual(PyObject),
    LessThan(PyObject),
    LessThanOrEqual(PyObject),
    MultipleOf(PyObject),
    Pattern {
        match_func: PyObject,
        original_pattern: String,
    },
    // An arbitrary Python callable that accepts
    // a PyAny and returns a bool or errors
    Unknown {
        function: PyObject,
        error: PyObject,
    },
}

impl BuildValidator for Constraint {
    const EXPECTED_TYPE: &'static str = "constraint";

    fn build(
        schema: &PyDict,
        _config: Option<&PyDict>,
        _definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let constraint: &PyDict = schema.get_as_req(intern!(py, "constraint"))?;
        match constraint.get_as_req::<&str>(intern!(py, "type"))? {
            "min-length" => Ok(Constraint::MinLength(constraint.get_as_req(intern!(py, "value"))?).into()),
            "max-length" => Ok(Constraint::MaxLength(constraint.get_as_req(intern!(py, "value"))?).into()),
            "gt" => Ok(Constraint::GreaterThan(constraint.get_as_req(intern!(py, "value"))?).into()),
            "ge" => Ok(Constraint::GreaterThanOrEqual(constraint.get_as_req(intern!(py, "value"))?).into()),
            "lt" => Ok(Constraint::LessThan(constraint.get_as_req(intern!(py, "value"))?).into()),
            "le" => Ok(Constraint::LessThanOrEqual(constraint.get_as_req(intern!(py, "value"))?).into()),
            "multiple-of" => Ok(Constraint::MultipleOf(constraint.get_as_req(intern!(py, "value"))?).into()),
            "pattern" => {
                let pattern = constraint.get_as_req::<&str>(intern!(py, "value"))?;
                let regex_mod = py.import("re")?;
                let match_func = regex_mod
                    .getattr(intern!(py, "compile"))?
                    .call1((pattern,))?
                    .to_object(py)
                    .getattr(py, intern!(py, "match"))?;
                Ok(Constraint::Pattern {
                    match_func,
                    original_pattern: pattern.to_string(),
                }
                .into())
            }
            other => Err(
                SchemaError::new_err(format!("Unknown constraint type: {}", other)),
            )
        }
    }
}

fn py_obj_multiple_of(py: Python<'_>, multiple_of: &PyObject, value: &PyAny) -> PyResult<bool> {
    Ok(value
        .call_method1(intern!(py, "__mod__"), (multiple_of,))?
        .extract::<i64>()?
        == 0)
}

impl Validator for Constraint {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        _extra: &Extra,
        _definitions: &'data Definitions<CombinedValidator>,
        _recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        match self {
            Constraint::MinLength(min_length) => {
                let len = input.to_object(py).as_ref(py).len()?;
                match len < *min_length {
                    true => Err(crate::errors::ValError::new(
                        crate::errors::ErrorType::TooShort {
                            min_length: *min_length,
                            actual_length: len,
                        },
                        input,
                    )),
                    false => Ok(input.to_object(py)),
                }
            }
            Constraint::MaxLength(max_length) => {
                let len = input.to_object(py).as_ref(py).len()?;
                match len > *max_length {
                    true => Err(crate::errors::ValError::new(
                        crate::errors::ErrorType::TooLong {
                            max_length: *max_length,
                            actual_length: len,
                        },
                        input,
                    )),
                    false => Ok(input.to_object(py)),
                }
            }
            Constraint::GreaterThan(gt) => {
                if input
                    .to_object(py)
                    .as_ref(py)
                    .rich_compare(gt.as_ref(py), CompareOp::Gt)?
                    .is_true()?
                {
                    Ok(input.to_object(py))
                } else {
                    Err(crate::errors::ValError::new(
                        crate::errors::ErrorType::GreaterThan { gt: gt.extract(py)? },
                        input,
                    ))
                }
            }
            Constraint::GreaterThanOrEqual(ge) => {
                if input
                    .to_object(py)
                    .as_ref(py)
                    .rich_compare(ge.as_ref(py), CompareOp::Ge)?
                    .is_true()?
                {
                    Ok(input.to_object(py))
                } else {
                    Err(crate::errors::ValError::new(
                        crate::errors::ErrorType::GreaterThanEqual { ge: ge.extract(py)? },
                        input,
                    ))
                }
            }
            Constraint::LessThan(lt) => {
                if input
                    .to_object(py)
                    .as_ref(py)
                    .rich_compare(lt.as_ref(py), CompareOp::Lt)?
                    .is_true()?
                {
                    Ok(input.to_object(py))
                } else {
                    Err(crate::errors::ValError::new(
                        crate::errors::ErrorType::LessThan { lt: lt.extract(py)? },
                        input,
                    ))
                }
            }
            Constraint::LessThanOrEqual(le) => {
                if input
                    .to_object(py)
                    .as_ref(py)
                    .rich_compare(le.as_ref(py), CompareOp::Le)?
                    .is_true()?
                {
                    Ok(input.to_object(py))
                } else {
                    Err(crate::errors::ValError::new(
                        crate::errors::ErrorType::LessThanEqual { le: le.extract(py)? },
                        input,
                    ))
                }
            }
            Constraint::MultipleOf(multiple_of) => {
                if py_obj_multiple_of(py, multiple_of, input.to_object(py).as_ref(py))? {
                    Ok(input.to_object(py))
                } else {
                    Err(crate::errors::ValError::new(
                        crate::errors::ErrorType::MultipleOf {
                            multiple_of: multiple_of.extract(py)?,
                        },
                        input,
                    ))
                }
            }
            Constraint::Pattern {
                match_func,
                original_pattern,
            } => {
                let pattern = match_func.call1(py, (input.to_object(py),))?.is_true(py)?;
                if pattern {
                    Ok(input.to_object(py))
                } else {
                    Err(crate::errors::ValError::new(
                        crate::errors::ErrorType::StringPatternMismatch {
                            pattern: original_pattern.clone(),
                        },
                        input,
                    ))
                }
            }
            Constraint::Unknown { function, error: _ } => {
                let result = function.call1(py, (input.to_object(py),))?;
                if result.is_true(py)? {
                    Ok(input.to_object(py))
                } else {
                    todo!() // return a ValueError with the given error as a line error
                }
            }
        }
    }

    fn complete(&mut self, _definitions: &DefinitionsBuilder<CombinedValidator>) -> PyResult<()> {
        Ok(())
    }

    fn different_strict_behavior(
        &self,
        _definitions: Option<&DefinitionsBuilder<CombinedValidator>>,
        _ultra_strict: bool,
    ) -> bool {
        false
    }

    fn get_name(&self) -> &str {
        match self {
            Constraint::MinLength(_) => "min_length",
            Constraint::MaxLength(_) => "max_length",
            Constraint::GreaterThan(_) => "greater_than",
            Constraint::GreaterThanOrEqual(_) => "greater_than_or_equal",
            Constraint::LessThan(_) => "less_than",
            Constraint::LessThanOrEqual(_) => "less_than_or_equal",
            Constraint::MultipleOf(_) => "multiple_of",
            Constraint::Pattern { .. } => "pattern",
            Constraint::Unknown { .. } => "constraint",
        }
    }
}

impl PyGcTraverse for Constraint {
    fn py_gc_traverse(&self, visit: &pyo3::PyVisit<'_>) -> Result<(), pyo3::PyTraverseError> {
        match self {
            Constraint::GreaterThan(obj) => visit.call(obj),
            Constraint::GreaterThanOrEqual(obj) => visit.call(obj),
            Constraint::LessThan(obj) => visit.call(obj),
            Constraint::LessThanOrEqual(obj) => visit.call(obj),
            Constraint::MultipleOf(obj) => visit.call(obj),
            Constraint::Pattern { match_func, .. } => visit.call(match_func),
            Constraint::Unknown { function, .. } => visit.call(function),
            _ => Ok(()),
        }
    }
}
