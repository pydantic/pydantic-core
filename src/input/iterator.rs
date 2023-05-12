use super::Input;

use crate::errors::{ErrorType, ValError, ValLineError, ValResult};

pub fn calculate_output_init_capacity(iterator_size: Option<usize>, max_length: Option<usize>) -> usize {
    // The smaller number of either the input size or the max output length
    match (iterator_size, max_length) {
        (None, _) => 0,
        (Some(l), None) => l,
        (Some(l), Some(r)) => std::cmp::min(l, r),
    }
}

pub fn validate_with_errors<'a, 's, I, R, F, O>(
    iter: impl Iterator<Item = &'a I>,
    validation_func: &mut F,
    output_func: &mut O,
    min_length: Option<usize>,
    max_length: Option<usize>,
    field_type: &'static str,
    input: &'a impl Input<'a>,
) -> ValResult<'a, ()>
where
    F: FnMut(&'a I) -> ValResult<'a, R>,
    O: FnMut(R) -> ValResult<'a, usize>,
    I: Input<'a> + 'a,
{
    let mut errors: Vec<ValLineError> = Vec::new();
    let mut current_len = 0;
    let index = 0;
    for (index, item) in iter.enumerate() {
        match validation_func(item) {
            Ok(item) => {
                current_len = output_func(item)?;
                if let Some(max_length) = max_length {
                    if max_length <= current_len {
                        return Err(ValError::new(
                            ErrorType::TooLong {
                                field_type: field_type.to_string(),
                                max_length,
                                actual_length: index,
                            },
                            input,
                        ));
                    }
                }
            }
            Err(ValError::LineErrors(line_errors)) => {
                errors.extend(line_errors.into_iter().map(|err| err.with_outer_location(index.into())));
            }
            Err(ValError::Omit) => (),
            Err(err) => return Err(err),
        }
    }

    if let Some(min_length) = min_length {
        if min_length <= current_len {
            return Err(ValError::new(
                ErrorType::TooShort {
                    field_type: field_type.to_string(),
                    min_length,
                    actual_length: index,
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
