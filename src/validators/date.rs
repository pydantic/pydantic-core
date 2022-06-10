use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDate, PyDict};
use speedate::{Date, Time};
use strum::EnumMessage;

use crate::build_tools::{is_strict, SchemaDict};
use crate::errors::{as_internal, context, err_val_error, ErrorKind, InputValue, ValError, ValResult};
use crate::input::Input;

use super::{BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct DateValidator {
    strict: bool,
    le: Option<Date>,
    lt: Option<Date>,
    ge: Option<Date>,
    gt: Option<Date>,
}

impl BuildValidator for DateValidator {
    const EXPECTED_TYPE: &'static str = "date";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        _build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        Ok(Self {
            strict: is_strict(schema, config)?,
            le: py_date_as_date(schema, "le")?,
            lt: py_date_as_date(schema, "lt")?,
            ge: py_date_as_date(schema, "ge")?,
            gt: py_date_as_date(schema, "gt")?,
        }
        .into())
    }
}

impl Validator for DateValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        _extra: &Extra,
        _slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        let date = match self.strict {
            true => input.strict_date()?,
            false => {
                match input.lax_date() {
                    Ok(date) => date,
                    // if the date error was an internal error, return that immediately
                    Err(ValError::InternalErr(internal_err)) => return Err(ValError::InternalErr(internal_err)),
                    // otherwise, try creating a date from a datetime input
                    Err(date_err) => date_from_datetime(input, date_err)?,
                }
            }
        };
        self.validation_comparison(py, input, date)
    }

    fn validate_strict<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        _extra: &Extra,
        _slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        self.validation_comparison(py, input, input.strict_date()?)
    }

    fn get_name(&self, _py: Python) -> String {
        Self::EXPECTED_TYPE.to_string()
    }
}

impl DateValidator {
    fn validation_comparison<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        date: Date,
    ) -> ValResult<'data, PyObject> {
        macro_rules! check_constraint {
            ($constraint:ident, $error:path, $key:literal) => {
                if let Some(constraint) = &self.$constraint {
                    if !date.$constraint(constraint) {
                        return err_val_error!(
                            input_value = InputValue::InputRef(input),
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
        let py_date = PyDate::new(py, date.year as i32, date.month, date.day).map_err(as_internal)?;
        Ok(py_date.into_py(py))
    }
}

/// In lax mode, if the input is not a date, we try parsing the input as a datetime, then check it is an
/// "exact date", e.g. has a zero time component.
fn date_from_datetime<'data>(input: &'data dyn Input, date_err: ValError<'data>) -> ValResult<'data, Date> {
    let dt = match input.lax_datetime() {
        Ok(dt) => dt,
        Err(dt_err) => {
            return match dt_err {
                ValError::LineErrors(mut line_errors) => {
                    // if we got a errors while parsing the datetime,
                    // convert DateTimeParsing -> DateFromDatetimeParsing but keep the rest of the error unchanged
                    for line_error in line_errors.iter_mut() {
                        match line_error.kind {
                            ErrorKind::DateTimeParsing => {
                                line_error.kind = ErrorKind::DateFromDatetimeParsing;
                            }
                            _ => {
                                return Err(date_err);
                            }
                        }
                    }
                    Err(ValError::LineErrors(line_errors))
                }
                ValError::InternalErr(internal_err) => Err(ValError::InternalErr(internal_err)),
            };
        }
    };
    let zero_time = Time {
        hour: 0,
        minute: 0,
        second: 0,
        microsecond: 0,
    };
    if dt.time == zero_time && dt.offset.is_none() {
        Ok(dt.date)
    } else {
        err_val_error!(
            input_value = InputValue::InputRef(input),
            kind = ErrorKind::DateFromDatetimeInexact
        )
    }
}

fn py_date_as_date(schema: &PyDict, field: &str) -> PyResult<Option<Date>> {
    let py_date: Option<&PyDate> = schema.get_as(field)?;
    match py_date {
        Some(py_date) => {
            let date_str: &str = py_date.str()?.extract()?;
            match Date::parse_str(date_str) {
                Ok(date) => Ok(Some(date)),
                Err(err) => {
                    let error_description = err.get_documentation().unwrap_or_default();
                    let msg = format!("Unable to parse date {}, error: {}", date_str, error_description);
                    Err(PyValueError::new_err(msg))
                }
            }
        }
        None => Ok(None),
    }
}
