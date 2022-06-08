use pyo3::prelude::*;
use pyo3::types::{PyDate, PyDateTime, PyTime};
use speedate::{Date, DateTime};
use strum::EnumMessage;

use super::Input;
use crate::errors::{as_internal, context, err_val_error, ErrorKind, InputValue, ValResult};

#[inline]
pub fn str_as_bool<'a>(input: &'a dyn Input, str: &str) -> ValResult<'a, bool> {
    if str == "0"
        || str.eq_ignore_ascii_case("f")
        || str.eq_ignore_ascii_case("n")
        || str.eq_ignore_ascii_case("no")
        || str.eq_ignore_ascii_case("off")
        || str.eq_ignore_ascii_case("false")
    {
        Ok(false)
    } else if str == "1"
        || str.eq_ignore_ascii_case("t")
        || str.eq_ignore_ascii_case("y")
        || str.eq_ignore_ascii_case("on")
        || str.eq_ignore_ascii_case("yes")
        || str.eq_ignore_ascii_case("true")
    {
        Ok(true)
    } else {
        err_val_error!(input_value = InputValue::InputRef(input), kind = ErrorKind::BoolParsing)
    }
}

#[inline]
pub fn int_as_bool(input: &dyn Input, int: i64) -> ValResult<bool> {
    if int == 0 {
        Ok(false)
    } else if int == 1 {
        Ok(true)
    } else {
        err_val_error!(input_value = InputValue::InputRef(input), kind = ErrorKind::BoolParsing)
    }
}

#[inline]
pub fn str_as_int<'s, 'l>(input: &'s dyn Input, str: &'l str) -> ValResult<'s, i64> {
    if let Ok(i) = str.parse::<i64>() {
        Ok(i)
    } else if let Ok(f) = str.parse::<f64>() {
        float_as_int(input, f)
    } else {
        err_val_error!(input_value = InputValue::InputRef(input), kind = ErrorKind::IntParsing)
    }
}

pub fn float_as_int(input: &dyn Input, float: f64) -> ValResult<i64> {
    if float == f64::INFINITY {
        err_val_error!(
            input_value = InputValue::InputRef(input),
            kind = ErrorKind::IntNan,
            context = context!("nan_value" => "infinity")
        )
    } else if float == f64::NEG_INFINITY {
        err_val_error!(
            input_value = InputValue::InputRef(input),
            kind = ErrorKind::IntNan,
            context = context!("nan_value" => "negative infinity")
        )
    } else if float.is_nan() {
        err_val_error!(
            input_value = InputValue::InputRef(input),
            kind = ErrorKind::IntNan,
            context = context!("nan_value" => "NaN")
        )
    } else if float % 1.0 != 0.0 {
        err_val_error!(
            input_value = InputValue::InputRef(input),
            kind = ErrorKind::IntFromFloat
        )
    } else {
        Ok(float as i64)
    }
}

pub fn bytes_as_date<'a>(input: &'a dyn Input, bytes: &[u8]) -> ValResult<'a, Date> {
    match Date::parse_bytes(bytes) {
        Ok(date) => Ok(date),
        Err(err) => {
            err_val_error!(
                input_value = InputValue::InputRef(input),
                kind = ErrorKind::DateParsing,
                context = context!("parsing_error" => err.get_documentation().unwrap_or_default())
            )
        }
    }
}

pub fn bytes_as_datetime<'a, 'b>(input: &'a dyn Input, bytes: &'b [u8]) -> ValResult<'a, DateTime> {
    match DateTime::parse_bytes(bytes) {
        Ok(date) => Ok(date),
        Err(err) => {
            err_val_error!(
                input_value = InputValue::InputRef(input),
                kind = ErrorKind::DateTimeParsing,
                context = context!("parsing_error" => err.get_documentation().unwrap_or_default())
            )
        }
    }
}

pub fn int_as_datetime(input: &dyn Input, timestamp: i64, timestamp_microseconds: u32) -> ValResult<DateTime> {
    match DateTime::from_timestamp(timestamp, timestamp_microseconds) {
        Ok(date) => Ok(date),
        Err(err) => {
            err_val_error!(
                input_value = InputValue::InputRef(input),
                kind = ErrorKind::DateTimeParsing,
                context = context!("parsing_error" => err.get_documentation().unwrap_or_default())
            )
        }
    }
}

pub fn float_as_datetime(input: &dyn Input, timestamp: f64) -> ValResult<DateTime> {
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

pub fn date_from_datetime<'a>(input: &'a dyn Input, py: Python<'a>, dt: &'a PyDateTime) -> ValResult<'a, &'a PyDate> {
    // TODO replace all this with raw rust types once github.com/samuelcolvin/speedate#6 is done
    // we want to make sure the time is zero - e.g. the dt is an "exact date"

    let dt_time: &PyTime = dt
        .call_method0("time")
        .map_err(as_internal)?
        .extract()
        .map_err(as_internal)?;

    let zero_time = PyTime::new(py, 0, 0, 0, 0, None).map_err(as_internal)?;
    if dt_time.eq(zero_time).map_err(as_internal)? {
        dt.call_method0("date")
            .map_err(as_internal)?
            .extract()
            .map_err(as_internal)
    } else {
        err_val_error!(
            input_value = InputValue::InputRef(input),
            kind = ErrorKind::DateFromDatetimeInexact
        )
    }
}

macro_rules! date_as_py_date {
    ($py:ident, $date:ident) => {
        pyo3::types::PyDate::new($py, $date.year as i32, $date.month, $date.day).map_err(crate::errors::as_internal)
    };
}
pub(crate) use date_as_py_date;

macro_rules! datetime_as_py_datetime {
    ($py:ident, $datetime:ident) => {
        pyo3::types::PyDateTime::new(
            $py,
            $datetime.date.year as i32,
            $datetime.date.month,
            $datetime.date.day,
            $datetime.time.hour,
            $datetime.time.minute,
            $datetime.time.second,
            $datetime.time.microsecond,
            None,
        )
        .map_err(crate::errors::as_internal)
    };
}
pub(crate) use datetime_as_py_datetime;
