use std::error::Error;
use std::fmt;
use std::result::Result as StdResult;

use pyo3::prelude::*;

use super::line_error::ValLineError;

pub type ValResult<T> = StdResult<T, ValError>;

#[derive(Debug)]
pub enum ValError {
    LineErrors(Vec<ValLineError>),
    InternalErr(PyErr),
}

impl fmt::Display for ValError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValError::LineErrors(line_errors) => {
                write!(f, "Line errors: {:?}", line_errors)
            }
            ValError::InternalErr(err) => {
                write!(f, "Internal error: {}", err)
            }
        }
    }
}

impl Error for ValError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ValError::LineErrors(_errors) => None,
            ValError::InternalErr(err) => Some(err),
        }
    }
}

pub fn as_internal(err: PyErr) -> ValError {
    ValError::InternalErr(err)
}
