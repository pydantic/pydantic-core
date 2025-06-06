use pyo3::exceptions::PyValueError;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use pyo3::IntoPyObjectExt;
use speedate::{MicrosecondsPrecisionOverflowBehavior, Time};

use crate::build_tools::is_strict;
use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::Input;

use super::datetime::extract_microseconds_precision;
use super::datetime::TZConstraint;
use super::{BuildValidator, CombinedValidator, DefinitionsBuilder, ValidationState, Validator};

#[derive(Debug, Clone)]
pub struct TimeValidator {
    strict: bool,
    constraints: Option<TimeConstraints>,
    microseconds_precision: MicrosecondsPrecisionOverflowBehavior,
}

impl BuildValidator for TimeValidator {
    const EXPECTED_TYPE: &'static str = "time";

    fn build(
        schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let s = Self {
            strict: is_strict(schema, config)?,
            constraints: TimeConstraints::from_py(schema)?,
            microseconds_precision: extract_microseconds_precision(schema, config)?,
        };
        Ok(s.into())
    }
}

impl_py_gc_traverse!(TimeValidator {});

impl Validator for TimeValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        let time = input
            .validate_time(state.strict_or(self.strict), self.microseconds_precision)?
            .unpack(state);
        if let Some(constraints) = &self.constraints {
            let raw_time = time.as_raw()?;

            macro_rules! check_constraint {
                ($constraint:ident, $error:ident) => {
                    if let Some(constraint) = &constraints.$constraint {
                        if !raw_time.$constraint(constraint) {
                            return Err(ValError::new(
                                ErrorType::$error {
                                    $constraint: constraint.to_string().into(),
                                    context: None,
                                },
                                input,
                            ));
                        }
                    }
                };
            }

            check_constraint!(le, LessThanEqual);
            check_constraint!(lt, LessThan);
            check_constraint!(ge, GreaterThanEqual);
            check_constraint!(gt, GreaterThan);

            if let Some(ref tz_constraint) = constraints.tz {
                tz_constraint.tz_check(raw_time.tz_offset, input)?;
            }
        }
        Ok(time.into_py_any(py)?)
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}

fn convert_pytime(schema: &Bound<'_, PyDict>, key: &Bound<'_, PyString>) -> PyResult<Option<Time>> {
    match schema.get_item(key)? {
        Some(value) => match value.validate_time(false, MicrosecondsPrecisionOverflowBehavior::default()) {
            Ok(v) => Ok(Some(v.into_inner().as_raw()?)),
            Err(_) => Err(PyValueError::new_err(format!(
                "'{key}' must be coercible to a time instance",
            ))),
        },
        None => Ok(None),
    }
}

#[derive(Debug, Clone)]
struct TimeConstraints {
    le: Option<Time>,
    lt: Option<Time>,
    ge: Option<Time>,
    gt: Option<Time>,
    tz: Option<TZConstraint>,
}

impl TimeConstraints {
    fn from_py(schema: &Bound<'_, PyDict>) -> PyResult<Option<Self>> {
        let py = schema.py();
        let c = Self {
            le: convert_pytime(schema, intern!(py, "le"))?,
            lt: convert_pytime(schema, intern!(py, "lt"))?,
            ge: convert_pytime(schema, intern!(py, "ge"))?,
            gt: convert_pytime(schema, intern!(py, "gt"))?,
            tz: TZConstraint::from_py(schema)?,
        };
        if c.le.is_some() || c.lt.is_some() || c.ge.is_some() || c.gt.is_some() || c.tz.is_some() {
            Ok(Some(c))
        } else {
            Ok(None)
        }
    }
}
