use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::build_tools::{is_strict, SchemaDict};
use crate::errors::{LocItem, ValError, ValLineError};
use crate::input::Input;

use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, ValResult, Validator};

#[derive(Debug, Clone)]
pub struct UnionValidator {
    choices: Vec<CombinedValidator>,
}

impl BuildValidator for UnionValidator {
    const EXPECTED_TYPE: &'static str = "union";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let choices: Vec<CombinedValidator> = schema
            .get_as_req::<&PyList>("choices")?
            .iter()
            .map(|choice| build_validator(choice, config, build_context).map(|result| result.0))
            .collect::<PyResult<Vec<CombinedValidator>>>()?;

        if is_strict(schema, config)? {
            Ok(StrictUnionValidator { choices }.into())
        } else {
            Ok(Self { choices }.into())
        }
    }
}

impl Validator for UnionValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        // 1st pass: check if the value is an exact instance of one of the Union types
        if let Some(res) = self
            .choices
            .iter()
            .map(|validator| validator.validate_strict(py, input, extra, slots))
            .find(ValResult::is_ok)
        {
            return res;
        }

        let mut errors: Vec<ValLineError> = Vec::with_capacity(self.choices.len());

        // 2nd pass: check if the value can be coerced into one of the Union types
        for validator in &self.choices {
            let line_errors = match validator.validate(py, input, extra, slots) {
                Err(ValError::LineErrors(line_errors)) => line_errors,
                otherwise => return otherwise,
            };

            errors.extend(
                line_errors
                    .into_iter()
                    .map(|err| err.with_prefix_location(LocItem::S(validator.get_name(py)))),
            );
        }

        Err(ValError::LineErrors(errors))
    }

    fn get_name(&self, _py: Python) -> String {
        Self::EXPECTED_TYPE.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct StrictUnionValidator {
    choices: Vec<CombinedValidator>,
}

impl Validator for StrictUnionValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        let mut errors: Vec<ValLineError> = Vec::with_capacity(self.choices.len());

        for validator in &self.choices {
            let line_errors = match validator.validate_strict(py, input, extra, slots) {
                Err(ValError::LineErrors(line_errors)) => line_errors,
                otherwise => return otherwise,
            };

            errors.extend(
                line_errors
                    .into_iter()
                    .map(|err| err.with_prefix_location(LocItem::S(validator.get_name(py)))),
            );
        }

        Err(ValError::LineErrors(errors))
    }

    fn get_name(&self, _py: Python) -> String {
        "strict-union".to_string()
    }
}
