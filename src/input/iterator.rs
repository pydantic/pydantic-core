use pyo3::{PyErr, Python};

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
    pub min_length: usize,
    pub max_length: Option<usize>,
    pub max_input_length: Option<usize>,
}

pub struct IterableValidationChecks<'data> {
    input_length: usize,
    output_length: usize,
    fail_fast: bool,
    length_constraints: LengthConstraints,
    field_type: &'static str,
    errors: Vec<ValLineError<'data>>,
}

impl<'data> IterableValidationChecks<'data> {
    pub fn new(fail_fast: bool, length_constraints: LengthConstraints, field_type: &'static str) -> Self {
        Self {
            input_length: 0,
            output_length: 0,
            fail_fast,
            length_constraints,
            field_type,
            errors: vec![],
        }
    }
    pub fn add_error(&mut self, error: ValLineError<'data>) {
        self.errors.push(error)
    }
    pub fn filter_validation_result<R, I: Input<'data>>(
        &mut self,
        result: ValResult<'data, R>,
        input: &'data I,
    ) -> ValResult<'data, Option<R>> {
        let res = match result {
            Ok(v) => Ok(Some(v)),
            Err(ValError::LineErrors(line_errors)) => {
                if self.fail_fast {
                    Err(ValError::LineErrors(line_errors))
                } else {
                    self.errors.extend(line_errors);
                    Ok(None)
                }
            }
            Err(ValError::Omit) => Ok(None),
            Err(e) => Err(e),
        };
        self.input_length += 1;
        self.check_max_length(self.input_length, self.length_constraints.max_input_length, input)?;
        res
    }

    pub fn check_output_length<I: Input<'data>>(
        &mut self,
        output_length: usize,
        input: &'data I,
    ) -> ValResult<'data, ()> {
        self.output_length = output_length;
        self.check_max_length(
            output_length + self.errors.len(),
            self.length_constraints.max_length,
            input,
        )
    }

    pub fn finish<I: Input<'data>>(&mut self, input: &'data I) -> ValResult<'data, ()> {
        if self.length_constraints.min_length > self.output_length {
            let err = ValLineError::new(
                ErrorType::TooShort {
                    field_type: self.field_type.to_string(),
                    min_length: self.length_constraints.min_length,
                    actual_length: self.output_length,
                },
                input,
            );
            self.errors.push(err);
        }
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(ValError::LineErrors(std::mem::take(&mut self.errors)))
        }
    }

    #[inline(always)]
    fn check_max_length<I: Input<'data>>(
        &self,
        current_length: usize,
        max_length: Option<usize>,
        input: &'data I,
    ) -> ValResult<'data, ()> {
        if let Some(max_length) = max_length {
            if max_length < current_length {
                return Err(ValError::new(
                    ErrorType::TooLong {
                        field_type: self.field_type.to_string(),
                        max_length,
                        actual_length: current_length,
                    },
                    input,
                ));
            }
        }
        Ok(())
    }
}

pub fn map_iter_error<'data>(
    py: Python<'data>,
    input: &'data impl Input<'data>,
    loc: impl Into<LocItem>,
    err: PyErr,
) -> ValError<'data> {
    ValError::new_with_loc(
        ErrorType::IterationError {
            error: py_err_string(py, err),
        },
        input,
        loc.into(),
    )
}
