use pyo3::types::{PyDict, PyFrozenSet};
use pyo3::{ffi, prelude::*, AsPyPointer};

use super::any::AnyValidator;
use super::set::set_build;
use super::{build_validator, BuildValidator, CombinedValidator, Definitions, DefinitionsBuilder, Extra, Validator};
use crate::build_tools::SchemaDict;
use crate::errors::ErrorType;
use crate::errors::ValResult;
use crate::input::iterator::{
    validate_fallible_iterator, validate_infallible_iterator, IterableValidationChecks, LengthConstraints,
};
use crate::input::Input;
use crate::input::{py_error_on_minusone, GenericIterable};
use crate::recursion_guard::RecursionGuard;

use crate::errors::ValError;

#[derive(Debug, Clone)]
pub struct FrozenSetValidator {
    strict: bool,
    item_validator: Box<CombinedValidator>,
    min_length: usize,
    max_length: Option<usize>,
    generator_max_length: Option<usize>,
    name: String,
}

impl BuildValidator for FrozenSetValidator {
    const EXPECTED_TYPE: &'static str = "frozenset";
    set_build!();
}

fn frozen_set_add<K>(set: &PyFrozenSet, key: K) -> PyResult<()>
where
    K: ToPyObject,
{
    unsafe { py_error_on_minusone(set.py(), ffi::PySet_Add(set.as_ptr(), key.to_object(set.py()).as_ptr())) }
}

impl Validator for FrozenSetValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        definitions: &'data Definitions<CombinedValidator>,
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let create_err = |input| ValError::new(ErrorType::FrozenSetType, input);

        let field_type = "Frozenset";

        let generic_iterable = input.extract_iterable().map_err(|_| create_err(input))?;

        let strict = extra.strict.unwrap_or(self.strict);

        let length_constraints = LengthConstraints {
            min_length: self.min_length,
            max_length: self.max_length,
            max_input_length: self.generator_max_length,
        };

        let mut checks = IterableValidationChecks::new(false, length_constraints, field_type);

        let mut output = PyFrozenSet::empty(py)?;

        let len = |output: &&'data PyFrozenSet| output.len();
        let mut write = |output: &mut &'data PyFrozenSet, ob: PyObject| frozen_set_add(output, ob);

        match (generic_iterable, strict) {
            // Always allow actual lists or JSON arrays
            (GenericIterable::JsonArray(iter), _) => validate_infallible_iterator(
                py,
                input,
                extra,
                definitions,
                recursion_guard,
                &mut checks,
                iter.iter(),
                &self.item_validator,
                &mut output,
                &mut write,
                &len,
            )?,
            (GenericIterable::FrozenSet(iter), _) => validate_infallible_iterator(
                py,
                input,
                extra,
                definitions,
                recursion_guard,
                &mut checks,
                iter.iter(),
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
                Ok(iter) => validate_fallible_iterator(
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
