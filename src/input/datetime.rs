use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDate, PyDateTime, PyDelta, PyTzInfo};
use speedate::{Date, DateTime, Time};
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
                let py = py_date.py();
                Ok(Date {
                    year: py_date.getattr(intern!(py, "year"))?.extract()?,
                    month: py_date.getattr(intern!(py, "month"))?.extract()?,
                    day: py_date.getattr(intern!(py, "day"))?.extract()?,
                })
            }
        }
    }

    pub fn try_into_py(self, py: Python<'_>) -> PyResult<PyObject> {
        let date = match self {
            Self::Python(date) => Ok(date),
            Self::Speedate(date) => PyDate::new(py, date.year as i32, date.month, date.day),
        }?;
        Ok(date.into_py(py))
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
                let py = py_dt.py();

                let mut offset: Option<i32> = None;
                let tzinfo = py_dt.getattr(intern!(py, "tzinfo"))?;
                if !tzinfo.is_none() {
                    let offset_delta = tzinfo.getattr(intern!(py, "utcoffset"))?.call1((py_dt.as_ref(),))?;
                    // as per the docs, utcoffset() can return None
                    if !offset_delta.is_none() {
                        let offset_seconds: f64 =
                            offset_delta.getattr(intern!(py, "total_seconds"))?.call0()?.extract()?;
                        offset = Some(offset_seconds.round() as i32);
                    }
                }

                Ok(DateTime {
                    date: Date {
                        year: py_dt.getattr(intern!(py, "year"))?.extract()?,
                        month: py_dt.getattr(intern!(py, "month"))?.extract()?,
                        day: py_dt.getattr(intern!(py, "day"))?.extract()?,
                    },
                    time: Time {
                        hour: py_dt.getattr(intern!(py, "hour"))?.extract()?,
                        minute: py_dt.getattr(intern!(py, "minute"))?.extract()?,
                        second: py_dt.getattr(intern!(py, "second"))?.extract()?,
                        microsecond: py_dt.getattr(intern!(py, "microsecond"))?.extract()?,
                    },
                    offset,
                })
            }
        }
    }

    pub fn try_into_py(self, py: Python<'a>) -> PyResult<PyObject> {
        let dt = match self {
            Self::Speedate(datetime) => {
                let tz: Option<PyObject> = match datetime.offset {
                    Some(offset) => {
                        let tz_info = TzInfo::new(offset);
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
                )?
            }
            Self::Python(dt) => dt,
        };
        Ok(dt.into_py(py))
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
    // warning 0.1 is pluck from thin air to make it work
    // since an input of timestamp=1655205632.331557, gives microseconds=331557.035446167
    // it maybe need to be adjusted, up OR down
    if microseconds % 1.0 > 0.1 {
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
struct TzInfo {
    seconds: i32,
}

#[pymethods]
impl TzInfo {
    #[new]
    fn new(seconds: i32) -> Self {
        Self { seconds }
    }

    fn utcoffset<'p>(&self, py: Python<'p>, _dt: &PyDateTime) -> PyResult<&'p PyDelta> {
        PyDelta::new(py, 0, self.seconds, 0, true)
    }

    fn tzname(&self, _dt: &PyDateTime) -> String {
        self.__str__()
    }

    fn dst(&self, _dt: &PyDateTime) -> Option<&PyDelta> {
        None
    }

    fn __repr__(&self) -> String {
        format!("TzInfo({})", self.__str__())
    }

    fn __str__(&self) -> String {
        if self.seconds == 0 {
            "UTC".to_string()
        } else {
            let mins = self.seconds / 60;
            format!("{:+03}:{:02}", mins / 60, (mins % 60).abs())
        }
    }
}
