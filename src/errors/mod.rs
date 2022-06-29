use pyo3::prelude::*;

mod kinds;
mod line_error;
pub mod location;
mod validation_exception;

use crate::input::Input;

use self::location::{LocItem, Location};

pub use self::kinds::ErrorKind;
pub use self::line_error::{as_internal, pretty_line_errors, Context, InputValue, ValError, ValLineError, ValResult};
pub use self::validation_exception::ValidationError;

/// Utility for concisely creating a `ValLineError`
pub fn val_line_error<'d>(kind: ErrorKind, input: &'d impl Input<'d>, context: Option<Context>) -> ValLineError<'d> {
    ValLineError {
        kind,
        input_value: input.as_error_value(),
        context: match context {
            Some(context) => context,
            None => Context::default(),
        },
        reverse_location: Location::default(),
    }
}

pub fn val_line_error_loc<'d>(
    kind: ErrorKind,
    input: &'d impl Input<'d>,
    context: Option<Context>,
    loc: impl Into<LocItem>,
) -> ValLineError<'d> {
    ValLineError {
        kind,
        input_value: input.as_error_value(),
        context: match context {
            Some(context) => context,
            None => Context::default(),
        },
        reverse_location: vec![loc.into()],
    }
}

pub fn err_val_error<'d>(kind: ErrorKind, input: &'d impl Input<'d>, context: Option<Context>) -> ValError<'d> {
    ValError::LineErrors(vec![val_line_error(kind, input, context)])
}

pub fn err_val_error_loc<'d>(
    kind: ErrorKind,
    input: &'d impl Input<'d>,
    context: Option<Context>,
    loc: impl Into<LocItem>,
) -> ValError<'d> {
    ValError::LineErrors(vec![val_line_error_loc(kind, input, context, loc)])
}

macro_rules! context {
    ($($k:expr => $v:expr),* $(,)?) => {{
        Some(crate::errors::Context::new([$(($k.into(), $v.into()),)*]))
    }};
}
pub(crate) use context;

pub fn py_err_string(py: Python, err: PyErr) -> String {
    let value = err.value(py);
    match value.get_type().name() {
        Ok(type_name) => match value.str() {
            Ok(py_str) => {
                let str_cow = py_str.to_string_lossy();
                let str = str_cow.as_ref();
                if !str.is_empty() {
                    format!("{}: {}", type_name, str)
                } else {
                    type_name.to_string()
                }
            }
            Err(_) => format!("{}: <exception str() failed>", type_name),
        },
        Err(_) => "Unknown Error".to_string(),
    }
}
