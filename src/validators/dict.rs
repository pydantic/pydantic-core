use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::{is_strict, SchemaDict};
use crate::errors::{ErrorKind, ValError, ValLineError, ValResult};
use crate::input::{GenericMapping, Input};
use crate::recursion_guard::RecursionGuard;

use super::any::AnyValidator;
use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct DictValidator {
    strict: bool,
    key_validator: Box<CombinedValidator>,
    value_validator: Box<CombinedValidator>,
    min_items: Option<usize>,
    max_items: Option<usize>,
    name: String,
}

impl BuildValidator for DictValidator {
    const EXPECTED_TYPE: &'static str = "dict";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let key_validator = match schema.get_item(intern!(py, "keys_schema")) {
            Some(schema) => Box::new(build_validator(schema, config, build_context)?.0),
            None => Box::new(AnyValidator::build(schema, config, build_context)?),
        };
        let value_validator = match schema.get_item(intern!(py, "values_schema")) {
            Some(d) => Box::new(build_validator(d, config, build_context)?.0),
            None => Box::new(AnyValidator::build(schema, config, build_context)?),
        };
        let name = format!(
            "{}[{},{}]",
            Self::EXPECTED_TYPE,
            key_validator.get_name(),
            value_validator.get_name()
        );
        Ok(Self {
            strict: is_strict(schema, config)?,
            key_validator,
            value_validator,
            min_items: schema.get_as(intern!(py, "min_items"))?,
            max_items: schema.get_as(intern!(py, "max_items"))?,
            name,
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
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let dict = input.validate_dict(extra.strict.unwrap_or(self.strict))?;

        let mut op_len: Option<usize> = None;
        if let Some(min_length) = self.min_items {
            let len = dict.generic_len()?;
            if len < min_length {
                return Err(ValError::new(ErrorKind::TooShort { min_length }, input));
            }
            op_len = Some(len);
        }
        if let Some(max_length) = self.max_items {
            let len = match op_len {
                Some(len) => len,
                None => dict.generic_len()?,
            };
            if len > max_length {
                return Err(ValError::new(ErrorKind::TooLong { max_length }, input));
            }
        }

        match dict {
            GenericMapping::PyDict(py_dict) => {
                self.validate_generic_dict(py, py_dict.iter(), extra, slots, recursion_guard)
            }
            GenericMapping::PyGetAttr(_) => unreachable!(),
            GenericMapping::JsonObject(json_object) => {
                self.validate_generic_dict(py, json_object.iter(), extra, slots, recursion_guard)
            }
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn complete(&mut self, build_context: &BuildContext) -> PyResult<()> {
        self.key_validator.complete(build_context)?;
        self.value_validator.complete(build_context)
    }
}

impl DictValidator {
    fn validate_generic_dict<'s, 'data>(
        &'s self,
        py: Python<'data>,
        dict_iter: impl Iterator<Item = (&'data (impl Input<'data> + 'data), &'data (impl Input<'data> + 'data))>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let output = PyDict::new(py);
        let mut errors: Vec<ValLineError> = Vec::new();

        let key_validator = self.key_validator.as_ref();
        let value_validator = self.value_validator.as_ref();

        for (key, value) in dict_iter {
            let output_key = match key_validator.validate(py, key, extra, slots, recursion_guard) {
                Ok(value) => Some(value),
                Err(ValError::LineErrors(line_errors)) => {
                    for err in line_errors {
                        // these are added in reverse order so [key] is shunted along by the second call
                        errors.push(
                            err.with_outer_location("[key]".into())
                                .with_outer_location(key.as_loc_item()),
                        );
                    }
                    None
                }
                Err(err) => return Err(err),
            };
            let output_value = match value_validator.validate(py, value, extra, slots, recursion_guard) {
                Ok(value) => Some(value),
                Err(ValError::LineErrors(line_errors)) => {
                    for err in line_errors {
                        errors.push(err.with_outer_location(key.as_loc_item()));
                    }
                    None
                }
                Err(err) => return Err(err),
            };
            if let (Some(key), Some(value)) = (output_key, output_value) {
                output.set_item(key, value)?;
            }
        }

        if errors.is_empty() {
            Ok(output.into())
        } else {
            Err(ValError::LineErrors(errors))
        }
    }
}
