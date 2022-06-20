use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::{is_strict, SchemaDict};
use crate::errors::{as_internal, context, err_val_error, ErrorKind, ValError, ValLineError, ValResult};
use crate::input::{GenericMapping, Input, ToLocItem};

use super::{BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

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
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        Ok(Self {
            strict: is_strict(schema, config)?,
            key_validator_id: match schema.get_item("keys") {
                Some(schema) => build_context.add_unnamed_slot(schema, config)?,
                None => build_context.any_validator_id(),
            },
            value_validator_id: match schema.get_item("values") {
                Some(schema) => build_context.add_unnamed_slot(schema, config)?,
                None => build_context.any_validator_id(),
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
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
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
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        self._validation_logic(py, input, input.strict_dict()?, extra, slots)
    }

    fn get_name<'data>(&self, _py: Python, _slots: &'data [CombinedValidator]) -> String {
        Self::EXPECTED_TYPE.to_string()
    }
}

impl DictValidator {
    fn _validation_logic<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        dict: GenericMapping<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        if let Some(min_length) = self.min_items {
            if dict.generic_len() < min_length {
                return err_val_error!(
                    input_value = input.as_error_value(),
                    kind = ErrorKind::DictTooShort,
                    context = context!("min_length" => min_length)
                );
            }
        }
        if let Some(max_length) = self.max_items {
            if dict.generic_len() > max_length {
                return err_val_error!(
                    input_value = input.as_error_value(),
                    kind = ErrorKind::DictTooLong,
                    context = context!("max_length" => max_length)
                );
            }
        }
        let output = PyDict::new(py);
        let mut errors: Vec<ValLineError> = Vec::new();

        let key_validator = unsafe { slots.get_unchecked(self.key_validator_id) };
        let value_validator = unsafe { slots.get_unchecked(self.value_validator_id) };

        macro_rules! iter {
            ($dict:ident) => {
                for (key, value) in $dict.iter() {
                    let output_key = match key_validator.validate(py, key, extra, slots) {
                        Ok(value) => Some(value),
                        Err(ValError::LineErrors(line_errors)) => {
                            let loc = vec![key.to_loc(), "[key]".to_loc()];
                            for err in line_errors {
                                errors.push(err.with_prefix_location(&loc));
                            }
                            None
                        }
                        Err(err) => return Err(err),
                    };
                    let output_value = match value_validator.validate(py, value, extra, slots) {
                        Ok(value) => Some(value),
                        Err(ValError::LineErrors(line_errors)) => {
                            let loc = vec![key.to_loc()];
                            for err in line_errors {
                                errors.push(err.with_prefix_location(&loc));
                            }
                            None
                        }
                        Err(err) => return Err(err),
                    };
                    if let (Some(key), Some(value)) = (output_key, output_value) {
                        output.set_item(key, value).map_err(as_internal)?;
                    }
                }
            };
        }
        match dict {
            GenericMapping::PyDict(d) => iter!(d),
            GenericMapping::JsonObject(d) => iter!(d),
        }

        if errors.is_empty() {
            Ok(output.into())
        } else {
            Err(ValError::LineErrors(errors))
        }
    }
}
