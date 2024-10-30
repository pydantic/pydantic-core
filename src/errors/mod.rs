use crate::validators::ValidationState;
use pyo3::prelude::*;

mod line_error;
mod location;
mod types;
mod validation_exception;
mod value_exception;

pub use self::line_error::{InputValue, ToErrorValue, ValError, ValLineError, ValResult};
pub use self::location::LocItem;
pub use self::types::{list_all_errors, ErrorType, ErrorTypeDefaults, Number};
pub use self::validation_exception::ValidationError;
pub use self::value_exception::{PydanticCustomError, PydanticKnownError, PydanticOmit, PydanticUseDefault};

pub fn py_err_string(py: Python, err: PyErr) -> String {
    let value = err.value_bound(py);
    match value.get_type().qualname() {
        Ok(type_name) => match value.str() {
            Ok(py_str) => {
                let str_cow = py_str.to_string_lossy();
                let str = str_cow.as_ref();
                if !str.is_empty() {
                    format!("{type_name}: {str}")
                } else {
                    type_name.to_string()
                }
            }
            Err(_) => format!("{type_name}: <exception str() failed>"),
        },
        Err(_) => "Unknown Error".to_string(),
    }
}

/// If we're in `allow_partial` mode, whether all errors occurred in the last element of the input.
pub fn sequence_valid_as_partial(state: &ValidationState, input_length: usize, errors: &[ValLineError]) -> bool {
    if !state.extra().allow_partial {
        return false;
    }
    // for the error to be in the last element, the index of all errors must be `input_length - 1`
    let last_index = (input_length - 1) as i64;
    errors.iter().all(|error| {
        if let Some(LocItem::I(loc_index)) = error.last_loc_item() {
            *loc_index == last_index
        } else {
            false
        }
    })
}

/// If we're in `allow_partial` mode, whether all errors occurred in the last  value of the input.
pub fn mapping_valid_as_partial(
    state: &ValidationState,
    opt_last_key: Option<impl Into<LocItem>>,
    errors: &[ValLineError],
) -> bool {
    if !state.extra().allow_partial {
        return false;
    }
    let Some(last_key) = opt_last_key.map(Into::into) else {
        return false;
    };
    errors.iter().all(|error| {
        if let Some(loc_item) = error.last_loc_item() {
            loc_item == &last_key
        } else {
            false
        }
    })
}
