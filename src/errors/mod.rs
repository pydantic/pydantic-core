mod kinds;
mod line_error;
mod val_error;
mod validation_exception;

pub use self::kinds::ErrorKind;
pub use self::line_error::{Context, LocItem, Location, ValLineError};
pub use self::val_error::{as_internal, ValError, ValResult};
pub use self::validation_exception::{as_validation_err, ValidationError};

/// Utility for concisely creating a `ValLineError`
/// can either take just `py` and a `value` (the given value) in which case kind `ErrorKind::ValueError` is used as kind
/// e.g. `val_line_error!(py, "the value provided")`
/// or, `py`, `value` and a mapping of other attributes for `ValLineError`
/// e.g. `val_line_error!(py, "the value provided", kind=ErrorKind::ExtraForbidden, message="the message")`
macro_rules! val_line_error {
    ($py:ident, $input:expr) => {
        crate::errors::ValLineError {
            input_value: Some($input.into_py($py)),
            ..Default::default()
        }
    };

    ($py:ident, $input:expr, $($key:ident = $val:expr),+) => {
        crate::errors::ValLineError {
            input_value: Some($input.to_py($py)),
            $(
                $key: $val,
            )+
            ..Default::default()
        }
    };
}
pub(crate) use val_line_error;

/// Utility for concisely creating a `Err(ValError::LineErrors([?]))` containing a single `ValLineError`
/// Usage matches `val_line_error`
macro_rules! err_val_error {
    ($py:ident, $input:expr) => {
        Err(crate::errors::ValError::LineErrors(vec![crate::errors::val_line_error!($py, $input)]))
    };

    ($py:ident, $input:expr, $($key:ident = $val:expr),+) => {
        Err(crate::errors::ValError::LineErrors(vec![crate::errors::val_line_error!($py, $input, $($key = $val),+)]))
    };
}
pub(crate) use err_val_error;

macro_rules! context {
    ($($k:expr => $v:expr),*) => {{
        Some(crate::errors::Context::new([$(($k, $v),)*]))
    }};
}
pub(crate) use context;
