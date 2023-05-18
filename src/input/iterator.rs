use std::marker::PhantomData;

use pyo3::{PyObject, PyResult, Python};

use super::Input;

use crate::validators::Validator;
use crate::{
    definitions::Definitions,
    errors::{ErrorType, ValError, ValLineError, ValResult},
    recursion_guard::RecursionGuard,
    validators::CombinedValidator,
    validators::Extra,
};

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
}

pub struct IterableValidationChecks<'data, I> {
    output_length: usize,
    min_length: usize,
    max_length: Option<usize>,
    field_type: &'static str,
    errors: Vec<ValLineError<'data>>,
    p: PhantomData<I>,
}

impl<'data, I: Input<'data> + 'data> IterableValidationChecks<'data, I> {
    pub fn new(length_constraints: LengthConstraints, field_type: &'static str) -> Self {
        Self {
            output_length: 0,
            min_length: length_constraints.min_length,
            max_length: length_constraints.max_length,
            field_type,
            errors: vec![],
            p: PhantomData,
        }
    }
    pub fn add_error(&mut self, error: ValLineError<'data>) {
        self.errors.push(error)
    }
    pub fn filter_validation_result<R>(
        &mut self,
        result: ValResult<'data, R>,
        input: &'data I,
    ) -> ValResult<'data, Option<R>> {
        match result {
            Ok(v) => Ok(Some(v)),
            Err(ValError::LineErrors(line_errors)) => {
                self.errors.extend(line_errors);
                if let Some(max_length) = self.max_length {
                    self.check_max_length(self.output_length + self.errors.len(), max_length, input)?;
                }
                Ok(None)
            }
            Err(ValError::Omit) => Ok(None),
            Err(e) => Err(e),
        }
    }
    pub fn check_output_length(&mut self, output_length: usize, input: &'data I) -> ValResult<'data, ()> {
        self.output_length = output_length;
        if let Some(max_length) = self.max_length {
            self.check_max_length(output_length + self.errors.len(), max_length, input)?;
        }
        Ok(())
    }
    pub fn finish(&mut self, input: &'data I) -> ValResult<'data, ()> {
        if self.min_length > self.output_length {
            let err = ValLineError::new(
                ErrorType::TooShort {
                    field_type: self.field_type.to_string(),
                    min_length: self.min_length,
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
    fn check_max_length(&self, current_length: usize, max_length: usize, input: &'data I) -> ValResult<'data, ()> {
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
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
pub fn validate_iterator<'s, 'data, V, O, W, L, I>(
    py: Python<'data>,
    input: &'data I,
    extra: &'s Extra<'s>,
    definitions: &'data Definitions<CombinedValidator>,
    recursion_guard: &'s mut RecursionGuard,
    checks: &mut IterableValidationChecks<'data, I>,
    iter: impl Iterator<Item = ValResult<'data, &'data V>>,
    items_validator: &'s CombinedValidator,
    output: &mut O,
    write: &mut W,
    len: &L,
) -> ValResult<'data, ()>
where
    I: Input<'data> + 'data,
    V: Input<'data> + 'data,
    W: FnMut(&mut O, PyObject) -> PyResult<()>,
    L: Fn(&O) -> usize,
{
    for (index, result) in iter.enumerate() {
        let value = result?;
        let result = items_validator
            .validate(py, value, extra, definitions, recursion_guard)
            .map_err(|e| e.with_outer_location(index.into()));
        if let Some(value) = checks.filter_validation_result(result, input)? {
            write(output, value)?;
            checks.check_output_length(len(output), input)?;
        }
    }
    checks.finish(input)?;
    Ok(())
}
