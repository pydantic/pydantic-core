use crate::errors::{boxed_input, err_val_error, ErrorKind, ValResult};

#[inline]
pub fn str_as_bool<'py>(str: &str) -> ValResult<'py, bool> {
    let s_lower: String = str.chars().map(|c| c.to_ascii_lowercase()).collect();
    match s_lower.as_str() {
        "0" | "off" | "f" | "false" | "n" | "no" => Ok(false),
        "1" | "on" | "t" | "true" | "y" | "yes" => Ok(true),
        _ => err_val_error!(
            input_value = boxed_input!(str.to_string()),
            kind = ErrorKind::BoolParsing
        ),
    }
}

#[inline]
pub fn int_as_bool<'py>(int: i64) -> ValResult<'py, bool> {
    if int == 0 {
        Ok(false)
    } else if int == 1 {
        Ok(true)
    } else {
        err_val_error!(input_value = boxed_input!(int), kind = ErrorKind::BoolParsing)
    }
}
