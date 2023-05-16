use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::{is_strict, SchemaDict};

use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::iterator::IterableValidationChecks;
use crate::input::Input;
use crate::input::{iterator, GenericIterable};
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

const FIELD_TYPE: &str = "Dictionary";

fn validation_function<'s, 'data, K, V>(
    py: Python<'data>,
    extra: &'s Extra<'s>,
    definitions: &'data Definitions<CombinedValidator>,
    recursion_guard: &'s mut RecursionGuard,
    key_validator: &'s CombinedValidator,
    value_validator: &'s CombinedValidator,
    key: &'data K,
    value: &'data V,
) -> ValResult<'data, (PyObject, PyObject)>
where
    K: Input<'data>,
    V: Input<'data>,
{
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
}

fn validate_mapping<'s, 'data, K, V>(
    py: Python<'data>,
    input: &'data impl Input<'data>,
    extra: &'s Extra<'s>,
    definitions: &'data Definitions<CombinedValidator>,
    recursion_guard: &'s mut RecursionGuard,
    checks: &mut IterableValidationChecks<'data>,
    iter: impl Iterator<Item = PyResult<(&'data K, &'data V)>>,
    key_validator: &'s CombinedValidator,
    value_validator: &'s CombinedValidator,
    output: &'data PyDict,
) -> ValResult<'data, ()>
where
    K: Input<'data> + 'data,
    V: Input<'data> + 'data,
{
    for result in iter {
        let (key, value) = result?; // TODO: handle error
        let result = validation_function(
            py,
            extra,
            definitions,
            recursion_guard,
            key_validator,
            value_validator,
            key,
            value,
        );
        if let Some((key, value)) = checks.filter_validation_result(result, input)? {
            output.set_item(key, value)?;
        }
        checks.check_output_length(output.len(), input)?;
    }
    checks.finish(input)?;
    Ok(())
}

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
            min_length: self.min_length.unwrap_or_default(),
            max_length: self.max_length,
            max_input_length: None,
        };

        let mut checks = IterableValidationChecks::new(false, length_constraints, FIELD_TYPE);

        let output = PyDict::new(py);

        let generic_iterable = input
            .extract_iterable()
            .map_err(|_| ValError::new(ErrorType::DictType, input))?;
        match (generic_iterable, strict) {
            // Always allow actual dicts or JSON objects
            (GenericIterable::Dict(iter), _) => validate_mapping(
                py,
                input,
                extra,
                definitions,
                recursion_guard,
                &mut checks,
                iter.iter().map(Ok),
                &self.key_validator,
                &self.value_validator,
                output,
            )?,
            (GenericIterable::JsonObject(iter), _) => validate_mapping(
                py,
                input,
                extra,
                definitions,
                recursion_guard,
                &mut checks,
                iter.iter().map(|(k, v)| (k, v)).map(Ok),
                &self.key_validator,
                &self.value_validator,
                output,
            )?,
            // If we're not in strict mode, accept other iterables, equivalent to calling dict(thing)
            (generic_iterable, false) => match generic_iterable.into_mapping_items_iterator(py) {
                Ok(iter) => validate_mapping(
                    py,
                    input,
                    extra,
                    definitions,
                    recursion_guard,
                    &mut checks,
                    iter,
                    &self.key_validator,
                    &self.value_validator,
                    output,
                )?,
                Err(_) => return Err(ValError::new(ErrorType::DictType, input)),
            },
            _ => return Err(ValError::new(ErrorType::DictType, input)),
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
