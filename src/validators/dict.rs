use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::{is_strict, SchemaDict};
use crate::errors::py_err_string;
use crate::errors::LocItem;
use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::Input;
use crate::input::{iterator, AnyIterable, JsonInput};
use crate::recursion_guard::RecursionGuard;

use super::any::AnyValidator;
use super::{build_validator, BuildValidator, CombinedValidator, Definitions, DefinitionsBuilder, Extra, Validator};

#[derive(Debug, Clone)]
pub struct DictValidator {
    strict: bool,
    key_validator: Box<CombinedValidator>,
    value_validator: Box<CombinedValidator>,
    min_length: Option<usize>,
    max_length: Option<usize>,
    name: String,
}

impl BuildValidator for DictValidator {
    const EXPECTED_TYPE: &'static str = "dict";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let key_validator = match schema.get_item(intern!(py, "keys_schema")) {
            Some(schema) => Box::new(build_validator(schema, config, definitions)?),
            None => Box::new(AnyValidator::build(schema, config, definitions)?),
        };
        let value_validator = match schema.get_item(intern!(py, "values_schema")) {
            Some(d) => Box::new(build_validator(d, config, definitions)?),
            None => Box::new(AnyValidator::build(schema, config, definitions)?),
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
            min_length: schema.get_as(intern!(py, "min_length"))?,
            max_length: schema.get_as(intern!(py, "max_length"))?,
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
        definitions: &'data Definitions<CombinedValidator>,
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let strict = extra.strict.unwrap_or(self.strict);

        let field_type: &'static str = "Dictionary";

        macro_rules! validate_python {
            ($iter:expr) => {{
                let output = PyDict::new(py);

                let key_validator = self.key_validator.as_ref();
                let value_validator = self.value_validator.as_ref();

                let mut validation_func =
                    |_: LocItem, (key, value): (&'data PyAny, &'data PyAny)| -> ValResult<'data, _> {
                        let v_key = key_validator
                            .validate(py, key, extra, definitions, recursion_guard)
                            .map_err(|e| {
                                e.with_outer_location("[key]".into())
                                    .with_outer_location(key.as_loc_item())
                            })?;
                        let v_value = value_validator
                            .validate(py, value, extra, definitions, recursion_guard)
                            .map_err(|e| e.with_outer_location(key.as_loc_item()))?;
                        Ok((v_key, v_value))
                    };

                let mut output_func = |(key, value): (PyObject, PyObject)| -> ValResult<'data, usize> {
                    output.set_item(key, value)?;
                    Ok(output.len())
                };
                iterator::validate_with_errors(
                    py,
                    $iter.map(|result| result.map(|(k, v)| (k.as_loc_item(), (k, v)))),
                    &mut validation_func,
                    &mut output_func,
                    self.min_length,
                    self.max_length,
                    field_type,
                    input,
                )?;

                output
            }};
        }

        macro_rules! validate_json {
            ($iter:expr) => {{
                let output = PyDict::new(py);

                let key_validator = self.key_validator.as_ref();
                let value_validator = self.value_validator.as_ref();

                let mut validation_func = |_: LocItem,
                                           (key, value): (&'data String, &'data JsonInput)|
                 -> ValResult<'data, (PyObject, PyObject)> {
                    let v_key = key_validator
                        .validate(py, key, extra, definitions, recursion_guard)
                        .map_err(|e| {
                            e.with_outer_location("[key]".into())
                                .with_outer_location(key.as_loc_item())
                        })?;
                    let v_value = value_validator
                        .validate(py, value, extra, definitions, recursion_guard)
                        .map_err(|e| e.with_outer_location(key.as_loc_item()))?;
                    Ok((v_key, v_value))
                };

                let mut output_func = |(key, value): (PyObject, PyObject)| -> ValResult<'data, usize> {
                    output.set_item(key, value)?;
                    Ok(output.len())
                };
                iterator::validate_with_errors(
                    py,
                    $iter.map(|(k, v): &'data (String, JsonInput)| Ok((k.as_loc_item(), (k, v)))),
                    &mut validation_func,
                    &mut output_func,
                    self.min_length,
                    self.max_length,
                    field_type,
                    input,
                )?;
                output
            }};
        }

        let map_default_err = |_| ValError::new(ErrorType::DictType, input);
        let output = match (input.extract_iterable().map_err(map_default_err)?, strict) {
            (AnyIterable::JsonObject(iter), _) => validate_json!(iter.iter()),
            (AnyIterable::Dict(iter), _) => validate_python!(iter.iter().map(Ok)),
            (AnyIterable::Mapping(iter), false) => validate_python!(iter
                .items()
                .map_err(|e| ValError::new(
                    ErrorType::MappingType {
                        error: py_err_string(py, e).into()
                    },
                    input
                ))?
                .iter()
                .map_err(|e| ValError::new(
                    ErrorType::MappingType {
                        error: py_err_string(py, e).into()
                    },
                    input
                ))?
                .map(|r| r.and_then(|v| v.extract::<(&PyAny, &PyAny)>()))),
            (_, _) => return Err(ValError::new(ErrorType::DictType, input)),
        };

        Ok(output.into_py(py))
    }

    fn different_strict_behavior(
        &self,
        definitions: Option<&DefinitionsBuilder<CombinedValidator>>,
        ultra_strict: bool,
    ) -> bool {
        if ultra_strict {
            self.key_validator.different_strict_behavior(definitions, true)
                || self.value_validator.different_strict_behavior(definitions, true)
        } else {
            true
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn complete(&mut self, definitions: &DefinitionsBuilder<CombinedValidator>) -> PyResult<()> {
        self.key_validator.complete(definitions)?;
        self.value_validator.complete(definitions)
    }
}
