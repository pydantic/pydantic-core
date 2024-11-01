use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::DowncastError;
use pyo3::DowncastIntoError;

use jiter::JsonValue;

use crate::input::BorrowInput;
use crate::input::Input;

use super::location::{LocItem, Location};
use super::types::ErrorType;

pub type ValResult<T> = Result<T, ValError>;

pub trait ToErrorValue {
    fn to_error_value(&self) -> InputValue;
}

impl<'a, T: BorrowInput<'a>> ToErrorValue for T {
    fn to_error_value(&self) -> InputValue {
        Input::as_error_value(self.borrow_input())
    }
}

impl ToErrorValue for &'_ dyn ToErrorValue {
    fn to_error_value(&self) -> InputValue {
        (**self).to_error_value()
    }
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum ValError {
    LineErrors(Vec<ValLineError>),
    InternalErr(PyErr),
    Omit,
    UseDefault,
}

impl From<PyErr> for ValError {
    fn from(py_err: PyErr) -> Self {
        Self::InternalErr(py_err)
    }
}

impl From<DowncastError<'_, '_>> for ValError {
    fn from(py_downcast: DowncastError) -> Self {
        Self::InternalErr(PyTypeError::new_err(py_downcast.to_string()))
    }
}

impl From<DowncastIntoError<'_>> for ValError {
    fn from(py_downcast: DowncastIntoError) -> Self {
        Self::InternalErr(PyTypeError::new_err(py_downcast.to_string()))
    }
}

impl From<Vec<ValLineError>> for ValError {
    fn from(line_errors: Vec<ValLineError>) -> Self {
        Self::LineErrors(line_errors)
    }
}

impl ValError {
    pub fn new(error_type: ErrorType, input: impl ToErrorValue) -> ValError {
        Self::LineErrors(vec![ValLineError::new(error_type, input)])
    }

    pub fn new_with_loc(error_type: ErrorType, input: impl ToErrorValue, loc: impl Into<LocItem>) -> ValError {
        Self::LineErrors(vec![ValLineError::new_with_loc(error_type, input, loc)])
    }

    pub fn new_custom_input(error_type: ErrorType, input_value: InputValue) -> ValError {
        Self::LineErrors(vec![ValLineError::new_custom_input(error_type, input_value)])
    }

    /// helper function to call with_outer on line items if applicable
    pub fn with_outer_location(self, into_loc_item: impl Into<LocItem>) -> Self {
        let loc_item = into_loc_item.into();
        match self {
            Self::LineErrors(mut line_errors) => {
                for line_error in &mut line_errors {
                    line_error.location.with_outer(loc_item.clone());
                }
                Self::LineErrors(line_errors)
            }
            other => other,
        }
    }
}

/// A `ValLineError` is a single error that occurred during validation which is converted to a `PyLineError`
/// to eventually form a `ValidationError`.
/// I don't like the name `ValLineError`, but it's the best I could come up with (for now).
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ValLineError {
    pub error_type: ErrorType,
    // location is reversed so that adding an "outer" location item is pushing, it's reversed before showing to the user
    pub location: Location,
    pub input_value: InputValue,
}

impl ValLineError {
    pub fn new(error_type: ErrorType, input: impl ToErrorValue) -> ValLineError {
        Self {
            error_type,
            input_value: input.to_error_value(),
            location: Location::default(),
        }
    }

    pub fn new_with_loc(error_type: ErrorType, input: impl ToErrorValue, loc: impl Into<LocItem>) -> ValLineError {
        Self {
            error_type,
            input_value: input.to_error_value(),
            location: Location::new_some(loc.into()),
        }
    }

    pub fn new_with_full_loc(error_type: ErrorType, input: impl ToErrorValue, location: Location) -> ValLineError {
        Self {
            error_type,
            input_value: input.to_error_value(),
            location,
        }
    }

    pub fn new_custom_input(error_type: ErrorType, input_value: InputValue) -> ValLineError {
        Self {
            error_type,
            input_value,
            location: Location::default(),
        }
    }

    /// location is stored reversed so it's quicker to add "outer" items as that's what we always do
    /// hence `push` here instead of `insert`
    pub fn with_outer_location(mut self, into_loc_item: impl Into<LocItem>) -> Self {
        self.location.with_outer(into_loc_item.into());
        self
    }

    // change the error_type on a error in place
    pub fn with_type(mut self, error_type: ErrorType) -> Self {
        self.error_type = error_type;
        self
    }
}

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub enum InputValue {
    Python(PyObject),
    Json(JsonValue<'static>),
}

impl ToPyObject for InputValue {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            Self::Python(input) => input.clone_ref(py),
            Self::Json(input) => input.to_object(py),
        }
    }
}

pub struct LineErrorCollector {
    errors: Vec<ValLineError>,
    capacity: usize,
}

impl LineErrorCollector {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            capacity: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            // as this is used on the error pathway, avoid allocating until the first error
            errors: Vec::new(),
            capacity,
        }
    }

    pub fn ensure_empty(self) -> ValResult<()> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(ValError::LineErrors(self.errors))
        }
    }

    pub fn push(&mut self, error: ValLineError) {
        self.allocate_if_needed();
        self.errors.push(error);
    }

    fn collect(&mut self, errors: Vec<ValLineError>) {
        self.allocate_if_needed();
        self.errors.extend(errors);
    }

    fn allocate_if_needed(&mut self) {
        if self.errors.is_empty() && self.capacity > 0 {
            self.errors.reserve(self.capacity);
        }
    }
}

/// Helper trait only implemented for `ValResult` to allow chaining of `collect_line_error`
pub trait ValResultExt<T> {
    /// If `self` is an `Err`, collect the line errors into the `collector` and return the error.
    fn collect_line_errors(
        self,
        collector: &mut LineErrorCollector,
        location: impl Into<LocItem>,
    ) -> Result<Option<T>, ValidationControlFlow>;
}

impl<T> ValResultExt<T> for ValResult<T> {
    #[inline]
    fn collect_line_errors(
        self,
        collector: &mut LineErrorCollector,
        location: impl Into<LocItem>,
    ) -> Result<Option<T>, ValidationControlFlow> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(ValError::LineErrors(line_errors)) => {
                extend_collector(line_errors, collector, location.into());
                Ok(None)
            }
            Err(ValError::InternalErr(err)) => Err(ValidationControlFlow::InternalErr(err)),
            Err(ValError::Omit) => Err(ValidationControlFlow::Omit),
            Err(ValError::UseDefault) => Err(ValidationControlFlow::UseDefault),
        }
    }
}

#[cold]
fn extend_collector(line_errors: Vec<ValLineError>, collector: &mut LineErrorCollector, location: LocItem) {
    collector.collect(
        line_errors
            .into_iter()
            .map(|line_error| line_error.with_outer_location(location.clone()))
            .collect(),
    );
}

/// ValError, minus the LineErrors variant.
///
/// TODO: maybe rework ValError to contain this.
pub enum ValidationControlFlow {
    InternalErr(PyErr),
    Omit,
    UseDefault,
}

impl From<ValidationControlFlow> for ValError {
    fn from(control_flow: ValidationControlFlow) -> Self {
        match control_flow {
            ValidationControlFlow::InternalErr(err) => ValError::InternalErr(err),
            ValidationControlFlow::Omit => ValError::Omit,
            ValidationControlFlow::UseDefault => ValError::UseDefault,
        }
    }
}
