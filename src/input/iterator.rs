use pyo3::{PyObject, PyResult, Python};

use super::Input;

use crate::errors::{py_err_string, ErrorType, LocItem, ValError, ValLineError, ValResult};

pub fn calculate_output_init_capacity(iterator_size: Option<usize>, max_length: Option<usize>) -> usize {
    // The smaller number of either the input size or the max output length
    match (iterator_size, max_length) {
        (None, _) => 0,
        (Some(l), None) => l,
        (Some(l), Some(r)) => std::cmp::min(l, r),
    }
}

#[derive(Debug, Clone)]
pub struct LengthConstraints {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub max_input_length: Option<usize>,
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
pub fn validate_with_errors<'a, 's, I, R, F, O, E>(
    py: Python,
    iter: impl Iterator<Item = PyResult<(LocItem, I)>>,
    validation_func: &mut F,
    output_func: &mut O,
    length_constraints: LengthConstraints,
    field_type: &str,
    input: &'a impl Input<'a>,
    extras: &mut E,
    fail_fast: bool,
) -> ValResult<'a, ()>
where
    F: FnMut(&mut E, LocItem, I) -> ValResult<'a, R>,
    O: FnMut(R) -> ValResult<'a, usize>,
{
    let mut errors: Vec<ValLineError> = Vec::new();
    let mut current_len = 0;
    let mut error_count = 0;

    let check_max_length = |current_len: usize, max_length: Option<usize>| {
        if let Some(max_length) = max_length {
            if max_length < current_len {
                return Err(ValError::new(
                    ErrorType::TooLong {
                        field_type: field_type.to_string(),
                        max_length,
                        actual_length: current_len,
                    },
                    input,
                ));
            }
        }
        Ok(())
    };

    for (index, item_result) in iter.enumerate() {
        check_max_length(index, length_constraints.max_input_length)?;
        match item_result {
            Ok((loc, item)) => match validation_func(extras, loc, item) {
                Ok(item) => {
                    current_len = output_func(item)?;
                    check_max_length(current_len + error_count, length_constraints.max_length)?;
                }
                Err(ValError::LineErrors(line_errors)) => {
                    if fail_fast {
                        return Err(ValError::LineErrors(errors));
                    }
                    error_count += 1;
                    errors.extend(line_errors);
                    check_max_length(current_len + error_count, length_constraints.max_length)?;
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

    if let Some(min_length) = length_constraints.min_length {
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

/// Utility wrapper used by lists, sets and variable tuples
#[allow(clippy::too_many_arguments)]
pub fn validate_iterable<'s, 'data, I, V, E, O, S>(
    py: Python,
    iter: impl Iterator<Item = PyResult<I>>,
    validator_function: &mut V,
    output_func: &mut O,
    length_constraints: LengthConstraints,
    field_type: &str,
    input: &'data impl Input<'data>,
    extras: &mut E,
    make_output: impl Fn(usize) -> PyResult<S>,
    input_len: Option<usize>,
    fail_fast: bool,
) -> ValResult<'data, S>
where
    V: FnMut(&mut E, LocItem, I) -> ValResult<'data, PyObject>,
    O: FnMut(&mut S, PyObject) -> ValResult<'data, usize>,
{
    let init_capacity = calculate_output_init_capacity(input_len, length_constraints.max_length);
    let mut output = make_output(init_capacity)?;

    let mut output_func_wrapper = |ob: PyObject| -> ValResult<'data, usize> { output_func(&mut output, ob) };

    validate_with_errors(
        py,
        iter.enumerate()
            .map(|(idx, result)| result.map(|v| (LocItem::from(idx), v))),
        validator_function,
        &mut output_func_wrapper,
        length_constraints,
        field_type,
        input,
        extras,
        fail_fast,
    )?;

    Ok(output)
}

/// Utility wrapper used by dicts and mappings
#[allow(clippy::too_many_arguments)]
pub fn validate_mapping<'s, 'data, I, V, E, O, S>(
    py: Python,
    iter: impl Iterator<Item = PyResult<I>>,
    validator_function: &mut V,
    output_func: &mut O,
    length_constraints: LengthConstraints,
    field_type: &str,
    input: &'data impl Input<'data>,
    extras: &mut E,
    make_output: impl Fn(usize) -> PyResult<S>,
    input_len: Option<usize>,
    fail_fast: bool,
) -> ValResult<'data, S>
where
    V: FnMut(&mut E, LocItem, I) -> ValResult<'data, (PyObject, PyObject)>,
    O: FnMut(&mut S, (PyObject, PyObject)) -> ValResult<'data, usize>,
{
    let init_capacity = calculate_output_init_capacity(input_len, length_constraints.max_length);
    let mut output = make_output(init_capacity)?;

    let mut output_func_wrapper =
        |ob: (PyObject, PyObject)| -> ValResult<'data, usize> { output_func(&mut output, ob) };

    validate_with_errors(
        py,
        iter.enumerate()
            .map(|(idx, result)| result.map(|v| (LocItem::from(idx), v))),
        validator_function,
        &mut output_func_wrapper,
        length_constraints,
        field_type,
        input,
        extras,
        fail_fast,
    )?;

    Ok(output)
}
