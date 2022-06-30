use pyo3::prelude::*;
use pyo3::types::PyDict;
use speedate::Duration;

use crate::build_tools::{is_strict, SchemaDict};
use crate::errors::{as_internal, context, err_val_error, ErrorKind, ValResult};
use crate::input::{EitherTimedelta, Input};
use crate::recursion_guard::RecursionGuard;
use crate::SchemaError;

use super::{BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct TimedeltaValidator {
    strict: bool,
    constraints: Option<TimedeltaConstraints>,
}

#[derive(Debug, Clone)]
struct TimedeltaConstraints {
    le: Option<Duration>,
    lt: Option<Duration>,
    ge: Option<Duration>,
    gt: Option<Duration>,
}
impl BuildValidator for TimedeltaValidator {
    const EXPECTED_TYPE: &'static str = "timedelta";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        _build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let has_constraints = schema.get_item("le").is_some()
            || schema.get_item("lt").is_some()
            || schema.get_item("ge").is_some()
            || schema.get_item("gt").is_some();

        Ok(Self {
            strict: is_strict(schema, config)?,
            constraints: match has_constraints {
                true => Some(TimedeltaConstraints {
                    le: py_timedelta_as_timedelta(schema, "le")?,
                    lt: py_timedelta_as_timedelta(schema, "lt")?,
                    ge: py_timedelta_as_timedelta(schema, "ge")?,
                    gt: py_timedelta_as_timedelta(schema, "gt")?,
                }),
                false => None,
            },
        }
        .into())
    }
}

impl Validator for TimedeltaValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        _extra: &Extra,
        _slots: &'data [CombinedValidator],
        _recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let timedelta = match self.strict {
            true => input.strict_timedelta()?,
            false => input.lax_timedelta()?,
        };
        self.validation_comparison(py, input, timedelta)
    }

    fn validate_strict<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        _extra: &Extra,
        _slots: &'data [CombinedValidator],
        _recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        self.validation_comparison(py, input, input.strict_timedelta()?)
    }

    fn get_name(&self, _py: Python) -> String {
        Self::EXPECTED_TYPE.to_string()
    }
}

impl TimedeltaValidator {
    fn validation_comparison<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        timedelta: EitherTimedelta<'data>,
    ) -> ValResult<'data, PyObject> {
        if let Some(constraints) = &self.constraints {
            let raw_timedelta = timedelta.as_raw().map_err(as_internal)?;

            macro_rules! check_constraint {
                ($constraint:ident, $error:path, $key:literal) => {
                    if let Some(constraint) = &constraints.$constraint {
                        if !raw_timedelta.$constraint(constraint) {
                            return err_val_error!(
                                input_value = input.as_error_value(),
                                kind = $error,
                                context = context!($key => constraint.to_string())
                            );
                        }
                    }
                };
            }

            check_constraint!(le, ErrorKind::LessThanEqual, "le");
            check_constraint!(lt, ErrorKind::LessThan, "lt");
            check_constraint!(ge, ErrorKind::GreaterThanEqual, "ge");
            check_constraint!(gt, ErrorKind::GreaterThan, "gt");
        }
        timedelta.try_into_py(py).map_err(as_internal)
    }
}

fn py_timedelta_as_timedelta(schema: &PyDict, field: &str) -> PyResult<Option<Duration>> {
    match schema.get_as::<&PyAny>(field)? {
        Some(obj) => {
            let prefix = format!(r#"Invalid "{}" constraint for timedelta"#, field);
            let timedelta = obj
                .lax_timedelta()
                .map_err(|e| SchemaError::from_val_error(obj.py(), &prefix, e))?;
            Ok(Some(timedelta.as_raw()?))
        }
        None => Ok(None),
    }
}
