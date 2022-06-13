use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDate, PyDateTime, PyDelta, PyTzInfo};
use speedate::{Date, DateTime};
use strum::EnumMessage;

use super::Input;
use crate::errors::{context, err_val_error, ErrorKind, InputValue, ValResult};

pub enum EitherDate<'a> {
    Speedate(Date),
    Python(&'a PyDate),
}

impl<'a> EitherDate<'a> {
    pub fn as_speedate(&self) -> PyResult<Date> {
        match self {
            Self::Speedate(date) => Ok(date.clone()),
            Self::Python(py_date) => {
                let date_str: &str = py_date.str()?.extract()?;
                match Date::parse_str(date_str) {
                    Ok(date) => Ok(date),
                    Err(err) => {
                        let error_description = err.get_documentation().unwrap_or_default();
                        let msg = format!("Unable to parse date {}, error: {}", date_str, error_description);
                        Err(PyValueError::new_err(msg))
                    }
                }
            }
        }
    }

    pub fn as_python(&self, py: Python<'a>) -> PyResult<&'a PyDate> {
        match self {
            Self::Speedate(date) => PyDate::new(py, date.year as i32, date.month, date.day),
            Self::Python(date) => Ok(date),
        }
    }
}

pub enum EitherDateTime<'a> {
    Speedate(DateTime),
    Python(&'a PyDateTime),
}

impl<'a> EitherDateTime<'a> {
    pub fn as_speedate(&self) -> PyResult<DateTime> {
        match self {
            Self::Speedate(dt) => Ok(dt.clone()),
            Self::Python(py_dt) => {
                let dt_str: &str = py_dt.str()?.extract()?;
                match DateTime::parse_str(dt_str) {
                    Ok(dt) => Ok(dt),
                    Err(err) => {
                        let error_description = err.get_documentation().unwrap_or_default();
                        let msg = format!("Unable to parse datetime {}, error: {}", dt_str, error_description);
                        Err(PyValueError::new_err(msg))
                    }
                }
            }
        }
    }

    pub fn as_python(&self, py: Python<'a>) -> PyResult<&'a PyDateTime> {
        match self {
            Self::Speedate(datetime) => {
                let tz: Option<PyObject> = match datetime.offset {
                    Some(offset) => {
                        let tz_info = TzClass::new(offset);
                        Some(Py::new(py, tz_info)?.to_object(py))
                    }
                    None => None,
                };
                PyDateTime::new(
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
            }
            Self::Python(dt) => Ok(dt),
        }
    }
}

pub fn bytes_as_date<'a>(input: &'a dyn Input, bytes: &[u8]) -> ValResult<'a, EitherDate<'a>> {
    match Date::parse_bytes(bytes) {
        Ok(date) => Ok(EitherDate::Speedate(date)),
        Err(err) => {
            err_val_error!(
                input_value = InputValue::InputRef(input),
                kind = ErrorKind::DateParsing,
                context = context!("parsing_error" => err.get_documentation().unwrap_or_default())
            )
        }
    }
}

pub fn bytes_as_datetime<'a, 'b>(input: &'a dyn Input, bytes: &'b [u8]) -> ValResult<'a, EitherDateTime<'a>> {
    match DateTime::parse_bytes(bytes) {
        Ok(dt) => Ok(EitherDateTime::Speedate(dt)),
        Err(err) => {
            err_val_error!(
                input_value = InputValue::InputRef(input),
                kind = ErrorKind::DateTimeParsing,
                context = context!("parsing_error" => err.get_documentation().unwrap_or_default())
            )
        }
    }
}

pub fn int_as_datetime(input: &dyn Input, timestamp: i64, timestamp_microseconds: u32) -> ValResult<EitherDateTime> {
    match DateTime::from_timestamp(timestamp, timestamp_microseconds) {
        Ok(dt) => Ok(EitherDateTime::Speedate(dt)),
        Err(err) => {
            err_val_error!(
                input_value = InputValue::InputRef(input),
                kind = ErrorKind::DateTimeParsing,
                context = context!("parsing_error" => err.get_documentation().unwrap_or_default())
            )
        }
    }
}

pub fn float_as_datetime(input: &dyn Input, timestamp: f64) -> ValResult<EitherDateTime> {
    let microseconds = timestamp.fract().abs() * 1_000_000.0;
    if microseconds % 1.0 > 1e-3 {
        return err_val_error!(
            input_value = InputValue::InputRef(input),
            kind = ErrorKind::DateTimeParsing,
            // message copied from speedate
            context = context!("parsing_error" => "second fraction value is more than 6 digits long")
        );
    }
    int_as_datetime(input, timestamp.floor() as i64, microseconds as u32)
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
