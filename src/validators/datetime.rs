use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDateTime, PyDelta, PyDict, PyTzInfo};
use speedate::DateTime;
use strum::EnumMessage;

use crate::build_tools::{is_strict, SchemaDict};
use crate::errors::{as_internal, context, err_val_error, ErrorKind, InputValue, ValResult};
use crate::input::Input;

use super::{BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct DateTimeValidator {
    strict: bool,
    le: Option<DateTime>,
    lt: Option<DateTime>,
    ge: Option<DateTime>,
    gt: Option<DateTime>,
}

impl BuildValidator for DateTimeValidator {
    const EXPECTED_TYPE: &'static str = "datetime";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        _build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        Ok(Self {
            strict: is_strict(schema, config)?,
            le: py_datetime_as_datetime(schema, "le")?,
            lt: py_datetime_as_datetime(schema, "lt")?,
            ge: py_datetime_as_datetime(schema, "ge")?,
            gt: py_datetime_as_datetime(schema, "gt")?,
        }
        .into())
    }
}

impl Validator for DateTimeValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        _extra: &Extra,
        _slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        let date = match self.strict {
            true => input.strict_datetime()?,
            false => input.lax_datetime()?,
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
        self.validation_comparison(py, input, input.strict_datetime()?)
    }

    fn get_name(&self, _py: Python) -> String {
        Self::EXPECTED_TYPE.to_string()
    }
}

impl DateTimeValidator {
    fn validation_comparison<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        datetime: DateTime,
    ) -> ValResult<'data, PyObject> {
        macro_rules! check_constraint {
            ($constraint:ident, $error:path, $key:literal) => {
                if let Some(constraint) = &self.$constraint {
                    if !datetime.$constraint(constraint) {
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

        let tz: Option<PyObject> = match datetime.offset {
            Some(offset) => {
                let tz_info = TzClass::new(offset);
                Some(Py::new(py, tz_info).map_err(as_internal)?.to_object(py))
            },
            None => None,
        };
        let py_dt = PyDateTime::new(
            py,
            datetime.date.year as i32,
            datetime.date.month,
            datetime.date.day,
            datetime.time.hour,
            datetime.time.minute,
            datetime.time.second,
            datetime.time.microsecond,
            tz.as_ref(),
        )
        .map_err(as_internal)?;
        Ok(py_dt.into_py(py))
    }
}

fn py_datetime_as_datetime(schema: &PyDict, field: &str) -> PyResult<Option<DateTime>> {
    let py_dt: Option<&PyDateTime> = schema.get_as(field)?;
    match py_dt {
        Some(py_dt) => {
            let dt_str: &str = py_dt.str()?.extract()?;
            match DateTime::parse_str(dt_str) {
                Ok(date) => Ok(Some(date)),
                Err(err) => {
                    let error_description = err.get_documentation().unwrap_or_default();
                    let msg = format!("Unable to parse datetime {}, error: {}", dt_str, error_description);
                    Err(PyValueError::new_err(msg))
                }
            }
        }
        None => Ok(None),
    }
}

#[pyclass(module = "pydantic_core._pydantic_core", extends = PyTzInfo)]
#[derive(Debug, Clone)]
struct TzClass {
    seconds: i32,
}

#[pymethods]
impl TzClass {
    #[new]
    fn new(seconds: i32) -> Self {
        Self { seconds }
    }

    fn utcoffset<'p>(&self, py: Python<'p>, _dt: &PyDateTime) -> PyResult<&'p PyDelta> {
        PyDelta::new(py, 0, self.seconds, 0, true)
    }

    fn tzname(&self, _py: Python<'_>, _dt: &PyDateTime) -> String {
        if self.seconds == 0 {
            "UTC".to_string()
        } else {
            let mins = self.seconds / 60;
            format!("{:+03}:{:02}", mins / 60, (mins % 60).abs())
        }
    }

    fn dst(&self, _py: Python<'_>, _dt: &PyDateTime) -> Option<&PyDelta> {
        None
    }
}
