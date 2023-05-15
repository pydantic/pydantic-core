use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFrozenSet, PySet};

use crate::build_tools::SchemaDict;
use crate::errors::{ErrorType, ValResult};
use crate::input::iterator::LengthConstraints;
use crate::input::{
    iterator::IterableValidator,  iterator::IterableValidatorBuilder, GenericIterable, Input,
};
use crate::recursion_guard::RecursionGuard;

use super::list::get_items_schema;
use super::{BuildValidator, CombinedValidator, Definitions, DefinitionsBuilder, Extra, Validator};

use crate::errors::{LocItem, ValError};
use crate::input::JsonInput;

const MAX_LENGTH_GEN_MULTIPLE: usize = 10;

#[derive(Debug, Clone)]
struct IntoSetValidator {
    strict: bool,
    item_validator: Option<Box<CombinedValidator>>,
    min_length: Option<usize>,
    max_length: Option<usize>,
    generator_max_length: Option<usize>,
    name: String,
}


pub fn validate_into_set<'data, 'py, I, R, L, V, D, N>(
    py: Python<'py>,
    iterator: &mut IterableValidator<'data, 'py, I, R, L, V, D, N>,
) -> ValResult<'data, &'data PySet>
where
    L: Into<LocItem>,
    V: FnMut(Python<'py>, &L, R) -> ValResult<'data, N>,
    I: Iterator<Item = ValResult<'data, (L, R)>>,
    D: Input<'data>,
    N: ToPyObject,
{
    let output = PySet::empty(py)?;
    while let Some(result) = iterator.next(py, output.len()) {
        let (_, data) = result?;
        output.add(data)?;
    }
    Ok(output)
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
        let item_validator = get_items_schema(schema, config, definitions)?;
        let inner_name = item_validator.as_ref().map(|v| v.get_name()).unwrap_or("any");
        let max_length = schema.get_as(pyo3::intern!(py, "max_length"))?;
        let generator_max_length = match schema.get_as(pyo3::intern!(py, "generator_max_length"))? {
            Some(v) => Some(v),
            None => max_length.map(|v| v * MAX_LENGTH_GEN_MULTIPLE),
        };
        let name = format!("{}[{}]", expected_type, inner_name);
        Ok(Self {
            strict: crate::build_tools::is_strict(schema, config)?,
            item_validator,
            min_length: schema.get_as(pyo3::intern!(py, "min_length"))?,
            max_length,
            generator_max_length,
            name,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn validate_into_set<'data, 's: 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        definitions: &'data Definitions<CombinedValidator>,
        recursion_guard: &'data mut RecursionGuard,
        // small breakage of encapsulation to avoid a lot of code duplication / macros
        set_type: SetType,
    ) -> ValResult<'data, &'data PySet> {
        let create_err = |input| match set_type {
            SetType::FrozenSet => ValError::new(ErrorType::FrozenSetType, input),
            SetType::Set => ValError::new(ErrorType::SetType, input),
        };

        let generic_iterable = input
            .extract_iterable()
            .map_err(|_| create_err(input))?;

        let field_type = match set_type {
            SetType::FrozenSet => "Frozenset",
            SetType::Set => "Set",
        };

        let builder = IterableValidatorBuilder::new(
            field_type,
            LengthConstraints {
                min_length: self.min_length,
                max_length: self.max_length,
                max_input_length: self.generator_max_length,
            },
            false,
        );

        let python_validation_func =
            |py: Python<'data>, loc: &usize, input: &'data PyAny| -> ValResult<'data, PyObject> {
                match &self.item_validator {
                    Some(validator) => validator
                        .validate(py, input, extra, definitions, recursion_guard)
                        .map_err(|e| e.with_outer_location(loc.clone().into())),
                    None => Ok(input.to_object(py)),
                }
            };

        let strict = extra.strict.unwrap_or(self.strict);

        let output: &PySet = match (generic_iterable, strict, set_type) {
            // Always allow actual lists or JSON arrays
            (GenericIterable::JsonArray(iter), _, _) => {
                let mut iterator = builder.build(
                    iter.into_iter().enumerate().map(Ok),
                    |py: Python<'_>, loc: &usize, input: &JsonInput| -> ValResult<'data, PyObject> {
                        match &self.item_validator {
                            Some(validator) => validator
                                .validate(py, input, extra, definitions, recursion_guard)
                                .map_err(|e| e.with_outer_location(loc.clone().into())),
                            None => Ok(input.to_object(py)),
                        }
                    },
                    input,
                );
                validate_into_set(py, &mut iterator)?
            },
            (GenericIterable::Set(iter), _, SetType::Set) => {
                let mut iterator = builder.build(iter.into_iter().enumerate().map(Ok), python_validation_func, input);
                validate_into_set(py, &mut iterator)?
            },
            (GenericIterable::FrozenSet(iter), _, SetType::FrozenSet) => {
                let mut iterator = builder.build(iter.into_iter().enumerate().map(Ok), python_validation_func, input);
                validate_into_set(py, &mut iterator)?
            },
            // If not in strict mode we also accept any iterable except str, bytes or mappings
            // This may seem counterintuitive since a Mapping is a less generic type than an arbitrary
            // iterable (which we do accept) but doing something like `x: list[int] = {1: 'a'}` is commonly
            // a mistake, so we don't parse it by default
            (
                GenericIterable::String(_)
                | GenericIterable::Bytes(_)
                | GenericIterable::Dict(_)
                | GenericIterable::Mapping(_),
                false, _
            ) => return Err(create_err(input)),
            (generic_iterable, false, _) => match generic_iterable.into_sequence_iterator(py) {
                Ok(iter) => {
                    let mut index = 0..;
                    let iter = iter
                        .into_iter()
                        .map(|result| result.map_err(|e| ValError::from(e)).map(|v| (index.next().unwrap(), v)));
                    let mut iterator = builder.build(iter, python_validation_func, input);
                    validate_into_set(py, &mut iterator)?
                }
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
            match self.item_validator {
                Some(ref v) => v.different_strict_behavior(definitions, true),
                None => false,
            }
        } else {
            true
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn complete(&mut self, definitions: &DefinitionsBuilder<CombinedValidator>) -> PyResult<()> {
        match self.item_validator {
            Some(ref mut v) => v.complete(definitions),
            None => Ok(()),
        }
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
