use std::ops::RangeFrom;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};

use crate::build_tools::{is_strict, SchemaDict};
use crate::errors::{ErrorType, ValError, ValLineError, ValResult};
use crate::input::iterator::LengthConstraints;
use crate::input::JsonInput;
use crate::input::{iterator::validate_into_vec, iterator::IterableValidatorBuilder, GenericIterable, Input};
use crate::recursion_guard::RecursionGuard;

use super::list::get_items_schema;
use super::{build_validator, BuildValidator, CombinedValidator, Definitions, DefinitionsBuilder, Extra, Validator};

#[derive(Debug, Clone)]
pub struct TupleVariableValidator {
    strict: bool,
    item_validator: Option<Box<CombinedValidator>>,
    min_length: Option<usize>,
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
        let item_validator = get_items_schema(schema, config, definitions)?;
        let inner_name = item_validator.as_ref().map(|v| v.get_name()).unwrap_or("any");
        let name = format!("tuple[{inner_name}, ...]");
        Ok(Self {
            strict: crate::build_tools::is_strict(schema, config)?,
            item_validator,
            min_length: schema.get_as(pyo3::intern!(py, "min_length"))?,
            max_length: schema.get_as(pyo3::intern!(py, "max_length"))?,
            name,
        }
        .into())
    }
}

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

        let builder = IterableValidatorBuilder::new(
            "Tuple",
            LengthConstraints {
                min_length: self.min_length,
                max_length: self.max_length,
                max_input_length: None,
            },
            false,
        );

        let python_validation_func =
            |py: Python<'data>, loc: &usize, input: &'data PyAny| -> ValResult<'data, PyObject> {
                match &self.item_validator {
                    Some(validator) => validator
                        .validate(py, input, extra, definitions, recursion_guard)
                        .map_err(|e| e.with_outer_location((*loc).into())),
                    None => Ok(input.to_object(py)),
                }
            };

        let strict = extra.strict.unwrap_or(self.strict);

        let output: Vec<PyObject> = match (generic_iterable, strict) {
            // Always allow actual lists or JSON arrays
            (GenericIterable::JsonArray(iter), _) => {
                let len = iter.len();
                let mut iterator = builder.build(
                    iter.iter().enumerate().map(Ok),
                    |py: Python<'_>, loc: &usize, value: &JsonInput| -> ValResult<'data, PyObject> {
                        match &self.item_validator {
                            Some(validator) => validator
                                .validate(py, value, extra, definitions, recursion_guard)
                                .map_err(|e| e.with_outer_location((*loc).into())),
                            None => Ok(input.to_object(py)),
                        }
                    },
                    input,
                );
                validate_into_vec(py, len, &mut iterator)?
            }
            (GenericIterable::Tuple(iter), _) => {
                let len = iter.len();
                let mut iterator = builder.build(iter.into_iter().enumerate().map(Ok), python_validation_func, input);
                validate_into_vec(py, len, &mut iterator)?
            }
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
                Ok(iter) => {
                    let mut index = 0..;
                    let capacity = iter.size_hint().1.unwrap_or(0);
                    let iter = iter
                        .into_iter()
                        .map(|result| result.map_err(ValError::from).map(|v| (index.next().unwrap(), v)));
                    let mut iterator = builder.build(iter, python_validation_func, input);
                    validate_into_vec(py, capacity, &mut iterator)?
                }
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
            match self.item_validator {
                Some(ref v) => v.different_strict_behavior(definitions, true),
                None => false,
            }
        } else {
            true
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn complete(&mut self, definitions: &DefinitionsBuilder<CombinedValidator>) -> PyResult<()> {
        match self.item_validator {
            Some(ref mut v) => v.complete(definitions),
            None => Ok(()),
        }
    }
}

#[derive(Debug, Clone)]
enum TuplePositionalItem<'s, 'data, T> {
    ValidatorAndItem((&'s CombinedValidator, &'data T)),
    ExtraItem((&'s CombinedValidator, &'data T)),
    DefaultValue(&'s CombinedValidator),
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

struct TuplePositionalInputIterator<'s, 'data, S, I> {
    input: &'data I,
    iter: S,
    num_item_validators: usize,
    item_validators: &'s [CombinedValidator],
    extra_validator: &'s Option<Box<CombinedValidator>>,
    current_index: RangeFrom<usize>,
}

impl<'s, 'data, S, I> TuplePositionalInputIterator<'s, 'data, S, I> {
    pub fn new(
        input: &'data I,
        iter: S,
        item_validators: &'s [CombinedValidator],
        extra_validator: &'s Option<Box<CombinedValidator>>,
    ) -> Self {
        Self {
            input,
            iter,
            num_item_validators: item_validators.len(),
            item_validators,
            extra_validator,
            current_index: 0..,
        }
    }
}

impl<'s, 'data, S, I, T: std::fmt::Debug + 'data> Iterator for TuplePositionalInputIterator<'s, 'data, S, I>
where
    S: Iterator<Item = ValResult<'data, &'data T>>,
    I: Input<'data>,
{
    type Item = ValResult<'data, (usize, TuplePositionalItem<'s, 'data, T>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.current_index.next().unwrap();
        match self.iter.next() {
            Some(result) => match result {
                Err(err) => Some(Err(err)),
                Ok(input) => {
                    match self.item_validators.get(0) {
                        Some(validator) => {
                            self.item_validators = &self.item_validators[1..];
                            Some(Ok((index, TuplePositionalItem::ValidatorAndItem((validator, input)))))
                        }
                        None => {
                            // Extra input item
                            match &self.extra_validator {
                                Some(validator) => {
                                    Some(Ok((index, TuplePositionalItem::ExtraItem((validator, input)))))
                                }
                                None => Some(Err(ValError::LineErrors(vec![ValLineError::new(
                                    ErrorType::TooLong {
                                        field_type: "Tuple".to_string(),
                                        max_length: self.num_item_validators,
                                        actual_length: index + 1,
                                    },
                                    self.input,
                                )]))),
                            }
                        }
                    }
                }
            },
            None => {
                // We've exhausted the input
                // Check if we still have any required slots to fill in and if so try to get those
                // from default values
                match self.item_validators.get(0) {
                    Some(validator) => {
                        self.item_validators = &self.item_validators[1..];
                        Some(Ok((index, TuplePositionalItem::DefaultValue(validator))))
                    }
                    None => None,
                }
            }
        }
    }
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

        let length_constraints = LengthConstraints {
            min_length: None, // we check this ourselves
            // If we don't have an extra_validator fail the length check
            max_length: None,
            max_input_length: None,
        };

        let builder = IterableValidatorBuilder::new("Tuple", length_constraints, false);

        let python_validation_func = |py: Python<'data>,
                                      loc: &usize,
                                      item: TuplePositionalItem<'s, 'data, PyAny>|
         -> ValResult<'data, PyObject> {
            let mut validate_inner =
                |validator: &'s CombinedValidator, maybe_item: Option<&'data PyAny>| match maybe_item {
                    Some(item) => validator
                        .validate(py, item, extra, definitions, recursion_guard)
                        .map_err(|e| e.with_outer_location((*loc).into())),
                    None => match validator.default_value(py, Some(*loc), extra, definitions, recursion_guard)? {
                        Some(default_value) => Ok(default_value),
                        None => Err(ValError::LineErrors(vec![ValLineError::new_with_loc(
                            ErrorType::Missing,
                            input,
                            *loc,
                        )])),
                    },
                };
            match item {
                TuplePositionalItem::ValidatorAndItem((validator, input)) => validate_inner(validator, Some(input)),
                TuplePositionalItem::ExtraItem((extra_validator, item)) => {
                    validate_inner(&(*extra_validator), Some(item))
                }
                TuplePositionalItem::DefaultValue(validator) => validate_inner(validator, None),
            }
        };

        let strict = extra.strict.unwrap_or(self.strict);

        let output: Vec<PyObject> = match (generic_iterable, strict) {
            // Always allow actual lists or JSON arrays
            (GenericIterable::JsonArray(iter), _) => {
                let capacity = iter.len();
                let input_iter = TuplePositionalInputIterator::new(
                    input,
                    iter.iter().map(Ok),
                    &self.items_validators,
                    &self.extra_validator,
                );
                let json_validation_func = |py: Python<'data>,
                                            loc: &usize,
                                            item: TuplePositionalItem<'s, 'data, JsonInput>|
                 -> ValResult<'data, PyObject> {
                    let mut validate_inner =
                        |validator: &'s CombinedValidator, maybe_item: Option<&'data JsonInput>| match maybe_item {
                            Some(item) => validator
                                .validate(py, item, extra, definitions, recursion_guard)
                                .map_err(|e| e.with_outer_location((*loc).into())),
                            None => {
                                match validator.default_value(py, Some(*loc), extra, definitions, recursion_guard)? {
                                    Some(default_value) => Ok(default_value),
                                    None => Err(ValError::LineErrors(vec![ValLineError::new_with_loc(
                                        ErrorType::Missing,
                                        input,
                                        *loc,
                                    )])),
                                }
                            }
                        };
                    match item {
                        TuplePositionalItem::ValidatorAndItem((validator, item)) => {
                            validate_inner(validator, Some(item))
                        }
                        TuplePositionalItem::ExtraItem((extra_validator, item)) => {
                            validate_inner(&(*extra_validator), Some(item))
                        }
                        TuplePositionalItem::DefaultValue(validator) => validate_inner(validator, None),
                    }
                };
                let mut iterator = builder.build(input_iter, json_validation_func, input);

                validate_into_vec(py, capacity, &mut iterator)?
            }
            (GenericIterable::Tuple(iter), _) => {
                let capacity = iter.len();
                let input_iter = TuplePositionalInputIterator::new(
                    input,
                    iter.iter().map(Ok),
                    &self.items_validators,
                    &self.extra_validator,
                );
                let mut iterator = builder.build(input_iter, python_validation_func, input);
                validate_into_vec(py, capacity, &mut iterator)?
            }
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
                Ok(iter) => {
                    let capacity = iter.size_hint().1.unwrap_or(0);
                    let input_iter = TuplePositionalInputIterator::new(
                        input,
                        iter.map(|v| v.map_err(ValError::from)),
                        &self.items_validators,
                        &self.extra_validator,
                    );
                    let mut iterator = builder.build(input_iter, python_validation_func, input);
                    validate_into_vec(py, capacity, &mut iterator)?
                }
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
