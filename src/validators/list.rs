use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::SchemaDict;
use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::iterator::{calculate_output_init_capacity, validate_iterator, IterableValidationChecks};
use crate::input::Input;
use crate::input::{iterator::LengthConstraints, GenericIterable};
use crate::recursion_guard::RecursionGuard;

use super::any::AnyValidator;
use super::{build_validator, BuildValidator, CombinedValidator, Definitions, DefinitionsBuilder, Extra, Validator};

#[derive(Debug, Clone)]
pub struct ListValidator {
    strict: bool,
    _allow_any_iter: bool,
    item_validator: Box<CombinedValidator>,
    min_length: usize,
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
        let item_validator = match schema.get_item(pyo3::intern!(schema.py(), "items_schema")) {
            Some(d) => build_validator(d, config, definitions)?,
            None => CombinedValidator::Any(AnyValidator),
        };
        let inner_name = item_validator.get_name();
        let name = format!("{}[{inner_name}]", Self::EXPECTED_TYPE);
        Ok(Self {
            strict: crate::build_tools::is_strict(schema, config)?,
            _allow_any_iter: schema.get_as(pyo3::intern!(py, "allow_any_iter"))?.unwrap_or(false),
            item_validator: Box::new(item_validator),
            min_length: schema.get_as(pyo3::intern!(py, "min_length"))?.unwrap_or_default(),
            max_length: schema.get_as(pyo3::intern!(py, "max_length"))?,
            name,
        }
        .into())
    }
}

const FIELD_TYPE: &str = "List";

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

        let strict = extra.strict.unwrap_or(self.strict);

        let length_constraints = LengthConstraints {
            min_length: self.min_length,
            max_length: self.max_length,
            max_input_length: None,
        };

        let mut checks = IterableValidationChecks::new(false, length_constraints, FIELD_TYPE);

        let mut output: Vec<PyObject> =
            Vec::with_capacity(calculate_output_init_capacity(generic_iterable.len(), self.max_length));

        let len = |output: &Vec<PyObject>| output.len();
        let mut write = |output: &mut Vec<PyObject>, ob: PyObject| {
            output.push(ob);
            Ok(())
        };

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
                &mut write,
                &len,
            )?,
            (GenericIterable::List(iter), _) => validate_iterator(
                py,
                input,
                extra,
                definitions,
                recursion_guard,
                &mut checks,
                iter.iter().map(Ok),
                &self.item_validator,
                &mut output,
                &mut write,
                &len,
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
            ) => return Err(ValError::new(ErrorType::ListType, input)),
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
                    &mut write,
                    &len,
                )?,
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
            self.item_validator.different_strict_behavior(definitions, true)
        } else {
            true
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn complete(&mut self, definitions: &DefinitionsBuilder<CombinedValidator>) -> PyResult<()> {
        self.item_validator.complete(definitions)?;
        let inner_name = self.item_validator.get_name();
        self.name = format!("{}[{inner_name}]", Self::EXPECTED_TYPE);
        Ok(())
    }
}
