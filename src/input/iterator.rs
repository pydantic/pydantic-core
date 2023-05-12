use pyo3::{PyResult, Python};

use super::Input;

use crate::errors::{py_err_string, ErrorType, ValError, ValLineError, ValResult};

pub fn calculate_output_init_capacity(iterator_size: Option<usize>, max_length: Option<usize>) -> usize {
    // The smaller number of either the input size or the max output length
    match (iterator_size, max_length) {
        (None, _) => 0,
        (Some(l), None) => l,
        (Some(l), Some(r)) => std::cmp::min(l, r),
    }
}

/// Validate an iterator by applying `validation_func`
/// to each item and calling `output_func` with non-error, non-omitted items.
/// This handles all of the complexity of accumulating errors and handling omitted items.
/// Capacity checks are also handled by having `output_func` return the current size of
/// the output, either from calling some `.len()` on whatever is accumulating the output
/// or keeping track of the number of times it was called.
/// This is implemented this way to account for collections that may not increase
/// their size for each item in the input (e.g. a set).
#[allow(clippy::too_many_arguments)]
pub fn validate_with_errors<'a, 's, I, R, F, O>(
    py: Python,
    iter: impl Iterator<Item = PyResult<I>>,
    validation_func: &mut F,
    output_func: &mut O,
    min_length: Option<usize>,
    max_length: Option<usize>,
    field_type: &'static str,
    input: &'a impl Input<'a>,
) -> ValResult<'a, ()>
where
    F: FnMut(I) -> ValResult<'a, R>,
    O: FnMut(R) -> ValResult<'a, usize>,
{
    let mut errors: Vec<ValLineError> = Vec::new();
    let mut current_len = 0;
    let mut error_count = 0;
    for (index, item_result) in iter.enumerate() {
        match item_result {
            Ok(item) => match validation_func(item) {
                Ok(item) => {
                    current_len = output_func(item)?;
                    if let Some(max_length) = max_length {
                        if max_length < current_len + error_count {
                            return Err(ValError::new(
                                ErrorType::TooLong {
                                    field_type: field_type.to_string(),
                                    max_length,
                                    actual_length: current_len + error_count,
                                },
                                input,
                            ));
                        }
                    }
                }
                Err(ValError::LineErrors(line_errors)) => {
                    error_count += 1;
                    errors.extend(line_errors.into_iter().map(|err| err.with_outer_location(index.into())));
                    if let Some(max_length) = max_length {
                        if max_length < current_len + error_count {
                            return Err(ValError::new(
                                ErrorType::TooLong {
                                    field_type: field_type.to_string(),
                                    max_length,
                                    actual_length: current_len + error_count,
                                },
                                input,
                            ));
                        }
                    }
                }
                Err(ValError::Omit) => (),
                Err(err) => return Err(err),
            },
            Err(err) => {
                return Err(ValError::new_with_loc(
                    ErrorType::IterationError {
                        error: py_err_string(py, err),
                    },
                    input,
                    index,
                ))
            } // Iterator failed
        }
    }

    if let Some(min_length) = min_length {
        if min_length > current_len {
            return Err(ValError::new(
                ErrorType::TooShort {
                    field_type: field_type.to_string(),
                    min_length,
                    actual_length: current_len,
                },
                input,
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ValError::LineErrors(errors))
    }
}
