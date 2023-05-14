use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::{is_strict, SchemaDict};

use crate::errors::LocItem;
use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::Input;
use crate::input::{iterator, GenericIterable, JsonInput};
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

// Grouping of parameters that get passed around to reduce number of fn params
struct Extras<'s, 'data> {
    py: Python<'data>,
    extra: &'s Extra<'s>,
    definitions: &'data Definitions<CombinedValidator>,
    recursion_guard: &'s mut RecursionGuard,
}

const FIELD_TYPE: &str = "Dictionary";

impl Validator for DictValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &'s Extra<'s>,
        definitions: &'data Definitions<CombinedValidator>,
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let strict = extra.strict.unwrap_or(self.strict);

        let length_constraints = iterator::LengthConstraints {
            min_length: self.min_length,
            max_length: self.max_length,
            max_input_length: None,
        };

        let mut extras: Extras = Extras {
            py,
            extra,
            definitions,
            recursion_guard,
        };

        let make_output = |_capacity: usize| Ok(PyDict::new(py));

        let mut output_func = |output: &mut &PyDict, ob: (PyObject, PyObject)| -> ValResult<'data, usize> {
            let (key, value) = ob;
            output.set_item(key, value)?;
            Ok(output.len())
        };

        let mut json_validator_func = |extras: &mut Extras<'s, 'data>,
                                       _loc: LocItem,
                                       (key, value): &'data (String, JsonInput)|
         -> ValResult<'data, (PyObject, PyObject)> {
            let v_key = self
                .key_validator
                .validate(extras.py, key, extras.extra, extras.definitions, extras.recursion_guard)
                .map_err(|e| {
                    e.with_outer_location("[key]".into())
                        .with_outer_location(key.as_loc_item())
                })?;
            let v_value = self
                .value_validator
                .validate(
                    extras.py,
                    value,
                    extras.extra,
                    extras.definitions,
                    extras.recursion_guard,
                )
                .map_err(|e| e.with_outer_location(key.as_loc_item()))?;
            Ok((v_key, v_value))
        };

        let mut python_validator_func = |extras: &mut Extras<'s, 'data>,
                                         _loc: LocItem,
                                         (key, value): (&'data PyAny, &'data PyAny)|
         -> ValResult<'data, (PyObject, PyObject)> {
            let v_key = self
                .key_validator
                .validate(extras.py, key, extras.extra, extras.definitions, extras.recursion_guard)
                .map_err(|e| {
                    e.with_outer_location("[key]".into())
                        .with_outer_location(key.as_loc_item())
                })?;
            let v_value = self
                .value_validator
                .validate(
                    extras.py,
                    value,
                    extras.extra,
                    extras.definitions,
                    extras.recursion_guard,
                )
                .map_err(|e| e.with_outer_location(key.as_loc_item()))?;
            Ok((v_key, v_value))
        };

        let generic_iterable = input
            .extract_iterable()
            .map_err(|_| ValError::new(ErrorType::DictType, input))?;
        let output = match (generic_iterable, strict) {
            // Always allow actual dicts or JSON objects
            (GenericIterable::Dict(iter), _) => iterator::validate_mapping(
                py,
                iter.iter().map(Ok),
                &mut python_validator_func,
                &mut output_func,
                length_constraints,
                FIELD_TYPE,
                input,
                &mut extras,
                make_output,
                Some(iter.len()),
                false,
            ),
            (GenericIterable::JsonObject(iter), _) => iterator::validate_mapping(
                py,
                iter.iter().map(Ok),
                &mut json_validator_func,
                &mut output_func,
                length_constraints,
                FIELD_TYPE,
                input,
                &mut extras,
                make_output,
                Some(iter.len()),
                false,
            ),
            // If we're not in strict mode, accept other iterables, equivalent to calling dict(thing)
            (generic_iterable, false) => match generic_iterable.into_mapping_items_iterator(py) {
                Ok(iter) => {
                    let len = iter.size_hint().1;
                    iterator::validate_mapping(
                        py,
                        iter,
                        &mut python_validator_func,
                        &mut output_func,
                        length_constraints,
                        FIELD_TYPE,
                        input,
                        &mut extras,
                        make_output,
                        len,
                        false,
                    )
                }
                Err(_) => return Err(ValError::new(ErrorType::DictType, input)),
            },
            _ => return Err(ValError::new(ErrorType::DictType, input)),
        }?;

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
