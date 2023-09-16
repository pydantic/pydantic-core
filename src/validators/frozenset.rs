use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFrozenSet};

use crate::errors::ValResult;
use crate::input::{GenericIterable, Input};
use crate::tools::SchemaDict;

use super::list::min_length_check;
use super::set::set_build;
use super::validation_state::ValidationState;
use super::{get_type, BuildValidator, CombinedValidator, DefinitionsBuilder, Validator, BUILTINS_TYPE};

#[derive(Debug, Clone)]
pub struct FrozenSetValidator {
    strict: bool,
    item_validator: Box<CombinedValidator>,
    min_length: Option<usize>,
    max_length: Option<usize>,
    name: String,
}

impl BuildValidator for FrozenSetValidator {
    const EXPECTED_TYPE: &'static str = "frozenset";
    set_build!();
}

impl_py_gc_traverse!(FrozenSetValidator { item_validator });

impl Validator for FrozenSetValidator {
    fn construct<'data>(
        &self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        state: &mut super::ConstructionState,
    ) -> ValResult<'data, PyObject> {
        if !state.recursive {
            return Ok(input.to_object(py));
        }

        let Ok(collection) = input.lax_frozenset() else {
            return Ok(input.to_object(py));
        };
        let output = collection.construct_to_vec(py, input, &self.item_validator, state)?;

        match collection {
            // In the cases below, copying is not really an option; so we instead return it as a concrete frozenset
            GenericIterable::Iterator(_)
            | GenericIterable::DictKeys(_)
            | GenericIterable::DictValues(_)
            | GenericIterable::DictItems(_) => Ok(PyFrozenSet::new(py, &output)?.into_py(py)),
            // Otherwise, we get the type of the input object and pass the output vector positionally to it's constructor
            // This causes issues if the input object does not implement a constructor that takes an iterable as it's
            // first argument, but I'm not sure we really have another good option to handle custom abstract classes
            // Alternatively, we could devise some other callback that a custom class must implement in order for
            // construct to work on it
            _ => {
                // return type(input)(output)
                let type_func = BUILTINS_TYPE.get_or_init(py, || get_type(py).unwrap());
                let input_type = type_func.call1(py, (input.to_object(py),))?;
                Ok(input_type.call1(py, (&output.into_py(py),))?.into_py(py))
            }
        }
    }

    fn validate<'data>(
        &self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        state: &mut ValidationState,
    ) -> ValResult<'data, PyObject> {
        let collection = input.validate_frozenset(state.strict_or(self.strict))?;
        let f_set = PyFrozenSet::empty(py)?;
        collection.validate_to_set(
            py,
            f_set,
            input,
            self.max_length,
            "Frozenset",
            &self.item_validator,
            state,
        )?;
        min_length_check!(input, "Frozenset", self.min_length, f_set);
        Ok(f_set.into_py(py))
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
