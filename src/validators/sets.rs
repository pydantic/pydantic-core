use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFrozenSet, PySet};

use crate::build_tools::SchemaDict;
use crate::errors::{ErrorType, ValResult};
use crate::input::iterator::{map_iter_error, IterableValidationChecks, LengthConstraints};
use crate::input::{GenericIterable, Input};
use crate::recursion_guard::RecursionGuard;

use super::any::AnyValidator;
use super::{build_validator, BuildValidator, CombinedValidator, Definitions, DefinitionsBuilder, Extra, Validator};

use crate::errors::ValError;

const MAX_LENGTH_GEN_MULTIPLE: usize = 10;

#[derive(Debug, Clone)]
struct IntoSetValidator {
    strict: bool,
    item_validator: Box<CombinedValidator>,
    min_length: usize,
    max_length: Option<usize>,
    generator_max_length: Option<usize>,
    name: String,
}

#[derive(Debug, Clone, Copy)]
enum SetType {
    FrozenSet,
    Set,
}

impl IntoSetValidator {
    pub fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        definitions: &mut DefinitionsBuilder<CombinedValidator>,
        expected_type: &str,
    ) -> PyResult<Self> {
        let py = schema.py();
        let item_validator = match schema.get_item(pyo3::intern!(schema.py(), "items_schema")) {
            Some(d) => build_validator(d, config, definitions)?,
            None => CombinedValidator::Any(AnyValidator),
        };
        let inner_name = item_validator.get_name();
        let max_length = schema.get_as(pyo3::intern!(py, "max_length"))?;
        let generator_max_length = match schema.get_as(pyo3::intern!(py, "generator_max_length"))? {
            Some(v) => Some(v),
            None => max_length.map(|v| v * MAX_LENGTH_GEN_MULTIPLE),
        };
        let name = format!("{}[{}]", expected_type, inner_name);
        Ok(Self {
            strict: crate::build_tools::is_strict(schema, config)?,
            item_validator: Box::new(item_validator),
            min_length: schema.get_as(pyo3::intern!(py, "min_length"))?.unwrap_or_default(),
            max_length,
            generator_max_length,
            name,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn validate_into_set<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        definitions: &'data Definitions<CombinedValidator>,
        recursion_guard: &'s mut RecursionGuard,
        // small breakage of encapsulation to avoid a lot of code duplication / macros
        set_type: SetType,
    ) -> ValResult<'data, &'data PySet> {
        let create_err = |input| match set_type {
            SetType::FrozenSet => ValError::new(ErrorType::FrozenSetType, input),
            SetType::Set => ValError::new(ErrorType::SetType, input),
        };

        let field_type = match set_type {
            SetType::FrozenSet => "Frozenset",
            SetType::Set => "Set",
        };

        let generic_iterable = input
            .extract_iterable()
            .map_err(|_| ValError::new(ErrorType::ListType, input))?;

        let strict = extra.strict.unwrap_or(self.strict);

        let length_constraints = LengthConstraints {
            min_length: self.min_length,
            max_length: self.max_length,
            max_input_length: self.generator_max_length,
        };

        let mut checks = IterableValidationChecks::new(false, length_constraints, field_type);

        let output = PySet::empty(py)?;

        match (generic_iterable, strict, set_type) {
            // Always allow actual lists or JSON arrays
            (GenericIterable::JsonArray(iter), _, _) => validate_iterator(
                py,
                input,
                extra,
                definitions,
                recursion_guard,
                &mut checks,
                iter.iter().map(Ok),
                &self.item_validator,
                output,
            )?,
            (GenericIterable::Set(iter), _, SetType::Set) => validate_iterator(
                py,
                input,
                extra,
                definitions,
                recursion_guard,
                &mut checks,
                iter.iter().map(Ok),
                &self.item_validator,
                output,
            )?,
            (GenericIterable::FrozenSet(iter), _, SetType::FrozenSet) => validate_iterator(
                py,
                input,
                extra,
                definitions,
                recursion_guard,
                &mut checks,
                iter.iter().map(Ok),
                &self.item_validator,
                output,
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
                _,
            ) => return Err(create_err(input)),
            (generic_iterable, false, _) => match generic_iterable.into_sequence_iterator(py) {
                Ok(iter) => validate_iterator(
                    py,
                    input,
                    extra,
                    definitions,
                    recursion_guard,
                    &mut checks,
                    iter,
                    &self.item_validator,
                    output,
                )?,
                Err(_) => return Err(create_err(input)),
            },
            _ => return Err(create_err(input)),
        };

        Ok(output)
    }

    pub fn different_strict_behavior(
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

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn complete(&mut self, definitions: &DefinitionsBuilder<CombinedValidator>) -> PyResult<()> {
        self.item_validator.complete(definitions)
    }
}

#[derive(Debug, Clone)]
pub struct FrozenSetValidator {
    inner: IntoSetValidator,
}

impl BuildValidator for FrozenSetValidator {
    const EXPECTED_TYPE: &'static str = "frozenset";
    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        Ok(Self {
            inner: IntoSetValidator::build(schema, config, definitions, Self::EXPECTED_TYPE)?,
        }
        .into())
    }
}

impl Validator for FrozenSetValidator {
    fn validate<'s, 'data, 'a>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &'a Extra<'a>,
        definitions: &'data Definitions<CombinedValidator>,
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let set = self
            .inner
            .validate_into_set(py, input, extra, definitions, recursion_guard, SetType::FrozenSet)?;
        Ok(PyFrozenSet::new(py, set)?.into_py(py))
    }

    fn different_strict_behavior(
        &self,
        definitions: Option<&DefinitionsBuilder<CombinedValidator>>,
        ultra_strict: bool,
    ) -> bool {
        self.inner.different_strict_behavior(definitions, ultra_strict)
    }

    fn get_name(&self) -> &str {
        self.inner.get_name()
    }

    fn complete(&mut self, definitions: &DefinitionsBuilder<CombinedValidator>) -> PyResult<()> {
        self.inner.complete(definitions)
    }
}

#[derive(Debug, Clone)]
pub struct SetValidator {
    inner: IntoSetValidator,
}

impl BuildValidator for SetValidator {
    const EXPECTED_TYPE: &'static str = "set";
    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        Ok(Self {
            inner: IntoSetValidator::build(schema, config, definitions, Self::EXPECTED_TYPE)?,
        }
        .into())
    }
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
        let set = self
            .inner
            .validate_into_set(py, input, extra, definitions, recursion_guard, SetType::Set)?;
        Ok(set.into_py(py))
    }

    fn different_strict_behavior(
        &self,
        definitions: Option<&DefinitionsBuilder<CombinedValidator>>,
        ultra_strict: bool,
    ) -> bool {
        self.inner.different_strict_behavior(definitions, ultra_strict)
    }

    fn get_name(&self) -> &str {
        self.inner.get_name()
    }

    fn complete(&mut self, definitions: &DefinitionsBuilder<CombinedValidator>) -> PyResult<()> {
        self.inner.complete(definitions)
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
    output: &PySet,
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
            output.add(value)?;
        }
        checks.check_output_length(output.len(), input)?;
    }
    checks.finish(input)?;
    Ok(())
}
