use speedate::Date;
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

pub fn string_as_date<'a>(input: &'a dyn Input, str: &str) -> ValResult<'a, Date> {
    match Date::parse_str(str) {
        Ok(date) => Ok(date),
        Err(err) => {
            let msg = err.get_documentation().unwrap_or_default();
            err_val_error!(
                input_value = InputValue::InputRef(input),
                kind = ErrorKind::DateParsing,
                context = context!("parsing_error" => msg)
            )
        }
    }
}

pub fn int_as_date<'a>(input: &'a dyn Input, int: i64) -> ValResult<'a, Date> {
    match Date::from_timestamp(int) {
        Ok(date) => Ok(date),
        Err(err) => {
            let msg = err.get_documentation().unwrap_or_default();
            err_val_error!(
                input_value = InputValue::InputRef(input),
                kind = ErrorKind::DateParsing,
                context = context!("parsing_error" => msg)
            )
        }
    }
}

macro_rules! date_as_py_date {
    ($py:ident, $date:ident) => {
        pyo3::types::PyDate::new($py, $date.year as i32, $date.month, $date.day).map_err(crate::errors::as_internal)
    };
}
pub(crate) use date_as_py_date;
