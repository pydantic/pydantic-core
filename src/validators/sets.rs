use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFrozenSet, PySet};

use crate::build_tools::SchemaDict;
use crate::errors::{ErrorType, ValResult};
use crate::input::Input;
use crate::recursion_guard::RecursionGuard;

use super::list::get_items_schema;
use super::{BuildValidator, CombinedValidator, Definitions, DefinitionsBuilder, Extra, Validator};

use crate::errors::{LocItem, ValError};
use crate::input::{iterator, GenericIterable, JsonInput};

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

// Grouping of parameters that get passed around to reduce number of fn params
struct Extras<'s, 'data> {
    py: Python<'data>,
    extra: &'s Extra<'s>,
    definitions: &'data Definitions<CombinedValidator>,
    recursion_guard: &'s mut RecursionGuard,
}

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
    pub fn validate_into_set<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &'s Extra<'s>,
        definitions: &'data Definitions<CombinedValidator>,
        recursion_guard: &'s mut RecursionGuard,
        // small breakage of encapsulation to avoid a lot of code duplication / macros
        set_type: SetType,
    ) -> ValResult<'data, &'data PySet> {
        let strict = extra.strict.unwrap_or(self.strict);

        let mut extras: Extras = Extras {
            py,
            extra,
            definitions,
            recursion_guard,
        };

        let length_constraints = iterator::LengthConstraints {
            min_length: self.min_length,
            max_length: self.max_length,
            max_input_length: self.generator_max_length,
        };

        let make_output = |_capacity: usize| PySet::new(py, Vec::<&PyAny>::new());

        let mut output_func = |output: &mut &PySet, ob: PyObject| -> ValResult<'data, usize> {
            output.add(ob)?;
            Ok(output.len())
        };

        let mut json_validator_func =
            |extras: &mut Extras<'s, 'data>, loc: LocItem, ob: &'data JsonInput| -> ValResult<'data, PyObject> {
                match &self.item_validator {
                    Some(v) => v
                        .validate(extras.py, ob, extras.extra, extras.definitions, extras.recursion_guard)
                        .map_err(|e| e.with_outer_location(loc)),
                    None => Ok(ob.to_object(py)),
                }
            };

        let mut python_validator_func =
            |extras: &mut Extras<'s, 'data>, loc: LocItem, ob: &'data PyAny| -> ValResult<'data, PyObject> {
                match &self.item_validator {
                    Some(v) => v
                        .validate(extras.py, ob, extras.extra, extras.definitions, extras.recursion_guard)
                        .map_err(|e| e.with_outer_location(loc)),
                    None => Ok(ob.to_object(py)),
                }
            };

        let (field_type, error_type) = match set_type {
            SetType::FrozenSet => ("Frozenset", ErrorType::FrozenSetType),
            SetType::Set => ("Set", ErrorType::SetType),
        };

        let generic_iterable = input
            .extract_iterable()
            .map_err(|_| ValError::new(error_type.clone(), input))?;
        let output = match (generic_iterable, strict, set_type) {
            // Always allow actual frozensets or JSON arrays
            (GenericIterable::JsonArray(iter), _, _) => iterator::validate_iterable(
                py,
                iter.iter().map(Ok),
                &mut json_validator_func,
                &mut output_func,
                length_constraints,
                field_type,
                input,
                &mut extras,
                make_output,
                Some(iter.len()),
                false,
            ),
            // See note above about accept_set_in_strict_mode and breaking encapsulation
            // accept_set_in_strict_mode is being used as a flag to determine if we should treat Set or FrozenSet as our "strict" type
            (GenericIterable::FrozenSet(iter), _, SetType::FrozenSet) => iterator::validate_iterable(
                py,
                iter.iter().map(Ok),
                &mut python_validator_func,
                &mut output_func,
                length_constraints,
                field_type,
                input,
                &mut extras,
                make_output,
                Some(iter.len()),
                false,
            ),
            (GenericIterable::Set(iter), _, SetType::Set) => iterator::validate_iterable(
                py,
                iter.iter().map(Ok),
                &mut python_validator_func,
                &mut output_func,
                length_constraints,
                field_type,
                input,
                &mut extras,
                make_output,
                Some(iter.len()),
                false,
            ),
            // If not in strict mode we also accept any iterable except str/bytes
            (GenericIterable::String(_) | GenericIterable::Bytes(_), _, _) => {
                return Err(ValError::new(error_type, input))
            }
            (generic_iterable, false, _) => match generic_iterable.into_sequence_iterator(py) {
                Ok(iter) => {
                    let len = iter.size_hint().1;
                    iterator::validate_iterable(
                        py,
                        iter,
                        &mut python_validator_func,
                        &mut output_func,
                        length_constraints,
                        field_type,
                        input,
                        &mut extras,
                        make_output,
                        len,
                        false,
                    )
                }
                Err(_) => return Err(ValError::new(error_type, input)),
            },
            _ => return Err(ValError::new(error_type, input)),
        }?;

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
