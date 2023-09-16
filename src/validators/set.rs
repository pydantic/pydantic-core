use pyo3::prelude::*;
use pyo3::types::{PyDict, PySet};

use crate::errors::ValResult;
use crate::input::{GenericIterable, Input};
use crate::tools::SchemaDict;

use super::list::min_length_check;
use super::{
    get_type, BuildValidator, CombinedValidator, DefinitionsBuilder, ValidationState, Validator, BUILTINS_TYPE,
};

#[derive(Debug, Clone)]
pub struct SetValidator {
    strict: bool,
    item_validator: Box<CombinedValidator>,
    min_length: Option<usize>,
    max_length: Option<usize>,
    name: String,
}

macro_rules! set_build {
    () => {
        fn build(
            schema: &PyDict,
            config: Option<&PyDict>,
            definitions: &mut DefinitionsBuilder<CombinedValidator>,
        ) -> PyResult<CombinedValidator> {
            let py = schema.py();
            let item_validator = match schema.get_item(pyo3::intern!(schema.py(), "items_schema")) {
                Some(d) => Box::new(crate::validators::build_validator(d, config, definitions)?),
                None => Box::new(crate::validators::any::AnyValidator::build(
                    schema,
                    config,
                    definitions,
                )?),
            };
            let inner_name = item_validator.get_name();
            let max_length = schema.get_as(pyo3::intern!(py, "max_length"))?;
            let name = format!("{}[{}]", Self::EXPECTED_TYPE, inner_name);
            Ok(Self {
                strict: crate::build_tools::is_strict(schema, config)?,
                item_validator,
                min_length: schema.get_as(pyo3::intern!(py, "min_length"))?,
                max_length,
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

impl_py_gc_traverse!(SetValidator { item_validator });

impl Validator for SetValidator {
    fn construct<'data>(
        &self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        state: &mut super::ConstructionState,
    ) -> ValResult<'data, PyObject> {
        if !state.recursive {
            return Ok(input.to_object(py));
        }

        let Ok(collection) = input.lax_set() else {
            return Ok(input.to_object(py));
        };
        let output = collection.construct_to_vec(py, input, &self.item_validator, state)?;

        match collection {
            // In the cases below, copying is not really an option; so we instead return it as a concrete set
            GenericIterable::Iterator(_)
            | GenericIterable::DictKeys(_)
            | GenericIterable::DictValues(_)
            | GenericIterable::DictItems(_) => Ok(PySet::new(py, &output)?.into_py(py)),
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
        let collection = input.validate_set(state.strict_or(self.strict))?;
        let set = PySet::empty(py)?;
        collection.validate_to_set(py, set, input, self.max_length, "Set", &self.item_validator, state)?;
        min_length_check!(input, "Set", self.min_length, set);
        Ok(set.into_py(py))
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
