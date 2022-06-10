use speedate::{Date, DateTime};
use strum::EnumMessage;

use super::Input;
use crate::errors::{context, err_val_error, ErrorKind, InputValue, ValResult};

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
