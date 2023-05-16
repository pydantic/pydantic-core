use pyo3::prelude::*;
use pyo3::types::{PyDict, PySet};

use super::{BuildValidator, Definitions, DefinitionsBuilder, Extra, Validator};
use crate::build_tools::SchemaDict;
use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::iterator::{validate_iterator, IterableValidationChecks, LengthConstraints};
use crate::input::{GenericIterable, Input};
use crate::recursion_guard::RecursionGuard;
use crate::validators::any::AnyValidator;
use crate::validators::{build_validator, CombinedValidator};

#[derive(Debug, Clone)]
pub struct SetValidator {
    strict: bool,
    item_validator: Box<CombinedValidator>,
    min_length: usize,
    max_length: Option<usize>,
    generator_max_length: Option<usize>,
    name: String,
}
pub static MAX_LENGTH_GEN_MULTIPLE: usize = 10;

macro_rules! set_build {
    () => {
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
            let max_length = schema.get_as(pyo3::intern!(py, "max_length"))?;
            let generator_max_length = match schema.get_as(pyo3::intern!(py, "generator_max_length"))? {
                Some(v) => Some(v),
                None => max_length.map(|v| v * super::set::MAX_LENGTH_GEN_MULTIPLE),
            };
            let name = format!("{}[{}]", Self::EXPECTED_TYPE, inner_name);
            Ok(Self {
                strict: crate::build_tools::is_strict(schema, config)?,
                item_validator: Box::new(item_validator),
                min_length: schema
                    .get_as(pyo3::intern!(py, "min_length"))?
                    .unwrap_or_default(),
                max_length,
                generator_max_length,
                name,
            }
            .into())
        }
    };
}
pub(crate) use set_build;

impl BuildValidator for SetValidator {
    const EXPECTED_TYPE: &'static str = "set";
    set_build!();
}

impl Validator for SetValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        definitions: &'data Definitions<CombinedValidator>,
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let create_err = |input| ValError::new(ErrorType::SetType, input);

        let field_type = "Set";

        let generic_iterable = input.extract_iterable().map_err(|_| create_err(input))?;

        let strict = extra.strict.unwrap_or(self.strict);

        let length_constraints = LengthConstraints {
            min_length: self.min_length,
            max_length: self.max_length,
            max_input_length: self.generator_max_length,
        };

        let mut checks = IterableValidationChecks::new(false, length_constraints, field_type);

        let mut output = PySet::empty(py)?;

        let len = |output: &&'data PySet| output.len();
        let mut write = |output: &mut &'data PySet, ob: PyObject| output.add(ob);

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
            (GenericIterable::Set(iter), _) => validate_iterator(
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
                false,
            ) => return Err(create_err(input)),
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
                Err(_) => return Err(create_err(input)),
            },
            _ => return Err(create_err(input)),
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
        self.item_validator.complete(definitions)
    }
}
