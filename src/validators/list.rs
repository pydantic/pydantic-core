use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::SchemaDict;
use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::Input;
use crate::input::{
    iterator::validate_into_vec, iterator::IterableValidatorBuilder, iterator::LengthConstraints, GenericIterable,
    JsonInput,
};
use crate::recursion_guard::RecursionGuard;

use super::{build_validator, BuildValidator, CombinedValidator, Definitions, DefinitionsBuilder, Extra, Validator};

#[derive(Debug, Clone)]
pub struct ListValidator {
    strict: bool,
    _allow_any_iter: bool,
    item_validator: Option<Box<CombinedValidator>>,
    min_length: Option<usize>,
    max_length: Option<usize>,
    name: String,
}

pub fn get_items_schema(
    schema: &PyDict,
    config: Option<&PyDict>,
    definitions: &mut DefinitionsBuilder<CombinedValidator>,
) -> PyResult<Option<Box<CombinedValidator>>> {
    match schema.get_item(pyo3::intern!(schema.py(), "items_schema")) {
        Some(d) => {
            let validator = build_validator(d, config, definitions)?;
            match validator {
                CombinedValidator::Any(_) => Ok(None),
                _ => Ok(Some(Box::new(validator))),
            }
        }
        None => Ok(None),
    }
}

impl BuildValidator for ListValidator {
    const EXPECTED_TYPE: &'static str = "list";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let item_validator = get_items_schema(schema, config, definitions)?;
        let inner_name = item_validator.as_ref().map(|v| v.get_name()).unwrap_or("any");
        let name = format!("{}[{inner_name}]", Self::EXPECTED_TYPE);
        Ok(Self {
            strict: crate::build_tools::is_strict(schema, config)?,
            _allow_any_iter: schema.get_as(pyo3::intern!(py, "allow_any_iter"))?.unwrap_or(false),
            item_validator,
            min_length: schema.get_as(pyo3::intern!(py, "min_length"))?,
            max_length: schema.get_as(pyo3::intern!(py, "max_length"))?,
            name,
        }
        .into())
    }
}

impl Validator for ListValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &'s Extra<'s>,
        definitions: &'data Definitions<CombinedValidator>,
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let generic_iterable = input
            .extract_iterable()
            .map_err(|_| ValError::new(ErrorType::ListType, input))?;

        let builder = IterableValidatorBuilder::new(
            "List",
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
                    |py: Python<'_>, loc: &usize, input: &JsonInput| -> ValResult<'data, PyObject> {
                        match &self.item_validator {
                            Some(validator) => validator
                                .validate(py, input, extra, definitions, recursion_guard)
                                .map_err(|e| e.with_outer_location((*loc).into())),
                            None => Ok(input.to_object(py)),
                        }
                    },
                    input,
                );
                validate_into_vec(py, len, &mut iterator)?
            }
            (GenericIterable::List(iter), _) => {
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
            ) => return Err(ValError::new(ErrorType::ListType, input)),
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
                Err(_) => return Err(ValError::new(ErrorType::ListType, input)),
            },
            _ => return Err(ValError::new(ErrorType::ListType, input)),
        };
        Ok(output.into_py(py))
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
        if let Some(ref mut v) = self.item_validator {
            v.complete(definitions)?;
            let inner_name = v.get_name();
            self.name = format!("{}[{inner_name}]", Self::EXPECTED_TYPE);
        }
        Ok(())
    }
}
