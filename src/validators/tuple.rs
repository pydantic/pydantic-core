use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};

use crate::build_tools::{is_strict, SchemaDict};
use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::iterator::calculate_output_init_capacity;
use crate::input::iterator::map_iter_error;
use crate::input::iterator::IterableValidationChecks;
use crate::input::iterator::LengthConstraints;
use crate::input::{GenericIterable, Input};
use crate::recursion_guard::RecursionGuard;

use super::any::AnyValidator;
use super::{build_validator, BuildValidator, CombinedValidator, Definitions, DefinitionsBuilder, Extra, Validator};

#[derive(Debug, Clone)]
pub struct TupleVariableValidator {
    strict: bool,
    item_validator: Box<CombinedValidator>,
    min_length: usize,
    max_length: Option<usize>,
    name: String,
}

impl BuildValidator for TupleVariableValidator {
    const EXPECTED_TYPE: &'static str = "tuple-variable";
    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let item_validator = match schema.get_item(pyo3::intern!(schema.py(), "items_schema")) {
            Some(d) => build_validator(d, config, definitions)?,
            None => CombinedValidator::Any(AnyValidator),
        };
        let inner_name = item_validator.get_name();
        let name = format!("tuple[{inner_name}, ...]");
        Ok(Self {
            strict: crate::build_tools::is_strict(schema, config)?,
            item_validator: Box::new(item_validator),
            min_length: schema.get_as(pyo3::intern!(py, "min_length"))?.unwrap_or_default(),
            max_length: schema.get_as(pyo3::intern!(py, "max_length"))?,
            name,
        }
        .into())
    }
}

fn validate_iterator<'s, 'data, V>(
    py: Python<'data>,
    input: &'data impl Input<'data>,
    extra: &'s Extra<'s>,
    definitions: &'data Definitions<CombinedValidator>,
    recursion_guard: &'s mut RecursionGuard,
    checks: &mut IterableValidationChecks<'data>,
    iter: impl Iterator<Item = PyResult<&'data V>>,
    items_validator: &'s CombinedValidator,
    output: &mut Vec<PyObject>,
) -> ValResult<'data, ()>
where
    V: Input<'data> + 'data,
{
    for (index, result) in iter.enumerate() {
        let value = result.map_err(|e| map_iter_error(py, input, index, e))?;
        let result = items_validator
            .validate(py, value, extra, definitions, recursion_guard)
            .map_err(|e| e.with_outer_location(index.into()));
        if let Some(value) = checks.filter_validation_result(result, input)? {
            output.push(value);
        }
        checks.check_output_length(output.len(), input)?;
    }
    checks.finish(input)?;
    Ok(())
}

const FIELD_TYPE: &str = "Tuple";

impl Validator for TupleVariableValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        definitions: &'data Definitions<CombinedValidator>,
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let generic_iterable = input
            .extract_iterable()
            .map_err(|_| ValError::new(ErrorType::TupleType, input))?;

        let strict = extra.strict.unwrap_or(self.strict);

        let length_constraints = LengthConstraints {
            min_length: self.min_length,
            max_length: self.max_length,
            max_input_length: None,
        };

        let mut checks = IterableValidationChecks::new(false, length_constraints, FIELD_TYPE);

        let mut output = Vec::with_capacity(calculate_output_init_capacity(generic_iterable.len(), self.max_length));

        match (generic_iterable, strict) {
            // Always allow actual lists or JSON arrays
            (GenericIterable::JsonArray(iter), _) => validate_iterator(
                py,
                input,
                extra,
                definitions,
                recursion_guard,
                &mut checks,
                iter.iter().map(Ok),
                &self.item_validator,
                &mut output,
            )?,
            (GenericIterable::Tuple(iter), _) => validate_iterator(
                py,
                input,
                extra,
                definitions,
                recursion_guard,
                &mut checks,
                iter.iter().map(Ok),
                &self.item_validator,
                &mut output,
            )?,
            // If not in strict mode we also accept any iterable except str, bytes or mappings
            // This may seem counterintuitive since a Mapping is a less generic type than an arbitrary
            // iterable (which we do accept) but doing something like `x: list[int] = {1: 'a'}` is commonly
            // a mistake, so we don't parse it by default
            (
                GenericIterable::String(_)
                | GenericIterable::Bytes(_)
                | GenericIterable::Dict(_)
                | GenericIterable::Mapping(_),
                _,
            ) => return Err(ValError::new(ErrorType::TupleType, input)),
            (generic_iterable, false) => match generic_iterable.into_sequence_iterator(py) {
                Ok(iter) => validate_iterator(
                    py,
                    input,
                    extra,
                    definitions,
                    recursion_guard,
                    &mut checks,
                    iter,
                    &self.item_validator,
                    &mut output,
                )?,
                Err(_) => return Err(ValError::new(ErrorType::TupleType, input)),
            },
            _ => return Err(ValError::new(ErrorType::TupleType, input)),
        };
        Ok(PyTuple::new(py, output.into_iter()).into_py(py))
    }

    fn different_strict_behavior(
        &self,
        definitions: Option<&DefinitionsBuilder<CombinedValidator>>,
        ultra_strict: bool,
    ) -> bool {
        if ultra_strict {
            self.item_validator.different_strict_behavior(definitions, true)
        } else {
            true
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn complete(&mut self, definitions: &DefinitionsBuilder<CombinedValidator>) -> PyResult<()> {
        self.item_validator.complete(definitions)
    }
}

#[derive(Debug, Clone)]
pub struct TuplePositionalValidator {
    strict: bool,
    items_validators: Vec<CombinedValidator>,
    extra_validator: Option<Box<CombinedValidator>>,
    name: String,
}

impl BuildValidator for TuplePositionalValidator {
    const EXPECTED_TYPE: &'static str = "tuple-positional";
    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let items: &PyList = schema.get_as_req(intern!(py, "items_schema"))?;
        let validators: Vec<CombinedValidator> = items
            .iter()
            .map(|item| build_validator(item, config, definitions))
            .collect::<PyResult<_>>()?;

        let descr = validators.iter().map(|v| v.get_name()).collect::<Vec<_>>().join(", ");
        Ok(Self {
            strict: is_strict(schema, config)?,
            items_validators: validators,
            extra_validator: match schema.get_item(intern!(py, "extra_schema")) {
                Some(v) => Some(Box::new(build_validator(v, config, definitions)?)),
                None => None,
            },
            name: format!("tuple[{descr}]"),
        }
        .into())
    }
}

fn validate_iterator_tuple_positional<'s, 'data, V>(
    py: Python<'data>,
    input: &'data impl Input<'data>,
    extra: &'s Extra<'s>,
    definitions: &'data Definitions<CombinedValidator>,
    recursion_guard: &'s mut RecursionGuard,
    checks: &mut IterableValidationChecks<'data>,
    iter: impl Iterator<Item = PyResult<&'data V>>,
    items_validators: &[CombinedValidator],
    extra_validator: &Option<Box<CombinedValidator>>,
    output: &mut Vec<PyObject>,
) -> ValResult<'data, ()>
where
    V: Input<'data> + 'data,
{
    for (index, result) in iter.enumerate() {
        let value = result.map_err(|e| map_iter_error(py, input, index, e))?;
        match items_validators.get(output.len()) {
            Some(item_validator) => {
                let result = item_validator
                    .validate(py, value, extra, definitions, recursion_guard)
                    .map_err(|e| e.with_outer_location(index.into()));
                if let Some(value) = checks.filter_validation_result(result, input)? {
                    output.push(value);
                }
            }
            None => {
                // Extra item
                match extra_validator {
                    Some(ref validator) => {
                        let result = validator
                            .validate(py, value, extra, definitions, recursion_guard)
                            .map_err(|e| e.with_outer_location(index.into()));
                        if let Some(value) = checks.filter_validation_result(result, input)? {
                            output.push(value);
                        }
                    }
                    None => {
                        return Err(ValError::new(
                            ErrorType::TooLong {
                                field_type: "Tuple".to_string(),
                                max_length: items_validators.len(),
                                actual_length: output.len() + 1,
                            },
                            input,
                        ))
                    }
                }
            }
        }
        checks.check_output_length(output.len(), input)?;
    }
    if output.len() < items_validators.len() {
        let remaining_item_validators = &items_validators[output.len()..];
        for validator in remaining_item_validators {
            let default = validator.default_value(py, Some(output.len()), extra, definitions, recursion_guard)?;
            match default {
                Some(v) => {
                    output.push(v);
                    checks.check_output_length(output.len(), input)?;
                }
                None => return Err(ValError::new_with_loc(ErrorType::Missing, input, output.len())),
            }
        }
    }
    checks.finish(input)?;
    Ok(())
}

impl Validator for TuplePositionalValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        definitions: &'data Definitions<CombinedValidator>,
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let generic_iterable = input
            .extract_iterable()
            .map_err(|_| ValError::new(ErrorType::TupleType, input))?;

        let strict = extra.strict.unwrap_or(self.strict);

        let length_constraints = LengthConstraints {
            min_length: 0,
            max_length: None,
            max_input_length: None,
        };

        let mut checks = IterableValidationChecks::new(false, length_constraints, FIELD_TYPE);

        let mut output = Vec::with_capacity(calculate_output_init_capacity(generic_iterable.len(), None));

        match (generic_iterable, strict) {
            // Always allow actual lists or JSON arrays
            (GenericIterable::JsonArray(iter), _) => validate_iterator_tuple_positional(
                py,
                input,
                extra,
                definitions,
                recursion_guard,
                &mut checks,
                iter.iter().map(Ok),
                &self.items_validators,
                &self.extra_validator,
                &mut output,
            )?,
            (GenericIterable::Tuple(iter), _) => validate_iterator_tuple_positional(
                py,
                input,
                extra,
                definitions,
                recursion_guard,
                &mut checks,
                iter.iter().map(Ok),
                &self.items_validators,
                &self.extra_validator,
                &mut output,
            )?,
            // If not in strict mode we also accept any iterable except str, bytes or mappings
            // This may seem counterintuitive since a Mapping is a less generic type than an arbitrary
            // iterable (which we do accept) but doing something like `x: list[int] = {1: 'a'}` is commonly
            // a mistake, so we don't parse it by default
            (
                GenericIterable::String(_)
                | GenericIterable::Bytes(_)
                | GenericIterable::Dict(_)
                | GenericIterable::Mapping(_),
                _,
            ) => return Err(ValError::new(ErrorType::TupleType, input)),
            (generic_iterable, false) => match generic_iterable.into_sequence_iterator(py) {
                Ok(iter) => validate_iterator_tuple_positional(
                    py,
                    input,
                    extra,
                    definitions,
                    recursion_guard,
                    &mut checks,
                    iter,
                    &self.items_validators,
                    &self.extra_validator,
                    &mut output,
                )?,
                Err(_) => return Err(ValError::new(ErrorType::TupleType, input)),
            },
            _ => return Err(ValError::new(ErrorType::TupleType, input)),
        };
        Ok(PyTuple::new(py, output.into_iter()).into_py(py))
    }

    fn different_strict_behavior(
        &self,
        definitions: Option<&DefinitionsBuilder<CombinedValidator>>,
        ultra_strict: bool,
    ) -> bool {
        if ultra_strict {
            if self
                .items_validators
                .iter()
                .any(|v| v.different_strict_behavior(definitions, true))
            {
                true
            } else if let Some(ref v) = self.extra_validator {
                v.different_strict_behavior(definitions, true)
            } else {
                false
            }
        } else {
            true
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn complete(&mut self, definitions: &DefinitionsBuilder<CombinedValidator>) -> PyResult<()> {
        self.items_validators
            .iter_mut()
            .try_for_each(|v| v.complete(definitions))?;
        match &mut self.extra_validator {
            Some(v) => v.complete(definitions),
            None => Ok(()),
        }
    }
}
