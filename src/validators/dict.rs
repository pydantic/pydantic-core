use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::{is_strict, SchemaDict};
use crate::errors::{as_internal, context, err_val_error, ErrorKind, InputValue, ValError, ValLineError, ValResult};
use crate::input::{DictInput, Input, ToLocItem};

use super::{BuildValidator, Extra, ValidateEnum, Validator, SlotsBuilder, get_validator};

#[derive(Debug, Clone)]
pub struct DictValidator {
    strict: bool,
    key_validator_id: usize,
    value_validator_id: usize,
    min_items: Option<usize>,
    max_items: Option<usize>,
    try_instance_as_dict: bool,
}

impl BuildValidator for DictValidator {
    const EXPECTED_TYPE: &'static str = "dict";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        slots_builder: &mut SlotsBuilder,
    ) -> PyResult<ValidateEnum> {
        Ok(Self {
            strict: is_strict(schema, config)?,
            key_validator_id: match schema.get_item("keys") {
                Some(schema) => slots_builder.build_add_anon(schema, config)?,
                None => slots_builder.get_any_validator(),
            },
            value_validator_id: match schema.get_item("values") {
                Some(schema) => slots_builder.build_add_anon(schema, config)?,
                None => slots_builder.get_any_validator(),
            },
            min_items: schema.get_as("min_items")?,
            max_items: schema.get_as("max_items")?,
            try_instance_as_dict: schema.get_as("try_instance_as_dict")?.unwrap_or(false),
        }
        .into())
    }
}

impl Validator for DictValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        extra: &Extra,
        slots: &'data [ValidateEnum],
    ) -> ValResult<'data, PyObject> {
        let dict = match self.strict {
            true => input.strict_dict()?,
            false => input.lax_dict(self.try_instance_as_dict)?,
        };
        self._validation_logic(py, input, dict, extra, slots)
    }

    fn validate_strict<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        extra: &Extra,
        slots: &'data [ValidateEnum],
    ) -> ValResult<'data, PyObject> {
        self._validation_logic(py, input, input.strict_dict()?, extra, slots)
    }

    fn get_name(&self, _py: Python) -> String {
        Self::EXPECTED_TYPE.to_string()
    }
}

impl DictValidator {
    fn _validation_logic<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        dict: Box<dyn DictInput<'data> + 'data>,
        extra: &Extra,
        slots: &'data [ValidateEnum],
    ) -> ValResult<'data, PyObject> {
        if let Some(min_length) = self.min_items {
            if dict.input_len() < min_length {
                return err_val_error!(
                    input_value = InputValue::InputRef(input),
                    kind = ErrorKind::DictTooShort,
                    context = context!("min_length" => min_length)
                );
            }
        }
        if let Some(max_length) = self.max_items {
            if dict.input_len() > max_length {
                return err_val_error!(
                    input_value = InputValue::InputRef(input),
                    kind = ErrorKind::DictTooLong,
                    context = context!("max_length" => max_length)
                );
            }
        }
        let output = PyDict::new(py);
        let mut errors: Vec<ValLineError> = Vec::new();

        for (key, value) in dict.input_iter() {
            let output_key: Option<PyObject> =
                apply_validator(py, self.key_validator_id, &mut errors, key, key, extra, slots, true)?;
            let output_value: Option<PyObject> =
                apply_validator(py, self.value_validator_id, &mut errors, value, key, extra, slots, false)?;
            if let (Some(key), Some(value)) = (output_key, output_value) {
                output.set_item(key, value).map_err(as_internal)?;
            }
        }

        if errors.is_empty() {
            Ok(output.into())
        } else {
            Err(ValError::LineErrors(errors))
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_validator<'s, 'data>(
    py: Python<'data>,
    validator_id: usize,
    errors: &mut Vec<ValLineError<'data>>,
    input: &'data dyn Input,
    key: &'data dyn Input,
    extra: &Extra,
    slots: &'data [ValidateEnum],
    key_loc: bool,
) -> ValResult<'data, Option<PyObject>> {
    let validator = get_validator(slots, validator_id)?;
    match validator.validate(py, input, extra, slots) {
        Ok(value) => Ok(Some(value)),
        Err(ValError::LineErrors(line_errors)) => {
            let loc = if key_loc {
                vec![key.to_loc(), "[key]".to_loc()]
            } else {
                vec![key.to_loc()]
            };
            for err in line_errors {
                errors.push(err.with_prefix_location(&loc));
            }
            Ok(None)
        }
        Err(err) => Err(err),
    }
}
