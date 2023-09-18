use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::errors::ValResult;
use crate::input::{GenericIterable, Input};
use crate::tools::SchemaDict;

use super::{
    build_validator, get_type, BuildValidator, CombinedValidator, ConstructionState, DefinitionsBuilder,
    ValidationState, Validator, BUILTINS_TYPE,
};

#[derive(Debug, Clone)]
pub struct ListValidator {
    strict: bool,
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

macro_rules! length_check {
    ($input:ident, $field_type:literal, $min_length:expr, $max_length:expr, $obj:ident) => {{
        let mut op_actual_length: Option<usize> = None;
        if let Some(min_length) = $min_length {
            let actual_length = $obj.len();
            if actual_length < min_length {
                return Err(crate::errors::ValError::new(
                    crate::errors::ErrorType::TooShort {
                        field_type: $field_type.to_string(),
                        min_length,
                        actual_length,
                        context: None,
                    },
                    $input,
                ));
            }
            op_actual_length = Some(actual_length);
        }
        if let Some(max_length) = $max_length {
            let actual_length = op_actual_length.unwrap_or_else(|| $obj.len());
            if actual_length > max_length {
                return Err(crate::errors::ValError::new(
                    crate::errors::ErrorType::TooLong {
                        field_type: $field_type.to_string(),
                        max_length,
                        actual_length,
                        context: None,
                    },
                    $input,
                ));
            }
        }
    }};
}
pub(crate) use length_check;

macro_rules! min_length_check {
    ($input:ident, $field_type:literal, $min_length:expr, $obj:ident) => {{
        if let Some(min_length) = $min_length {
            let actual_length = $obj.len();
            if actual_length < min_length {
                return Err(crate::errors::ValError::new(
                    crate::errors::ErrorType::TooShort {
                        field_type: $field_type.to_string(),
                        min_length,
                        actual_length,
                        context: None,
                    },
                    $input,
                ));
            }
        }
    }};
}
pub(crate) use min_length_check;

impl BuildValidator for ListValidator {
    const EXPECTED_TYPE: &'static str = "list";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let item_validator = get_items_schema(schema, config, definitions)?;
        let inner_name = item_validator.as_ref().map_or("any", |v| v.get_name());
        let name = format!("{}[{inner_name}]", Self::EXPECTED_TYPE);
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

impl_py_gc_traverse!(ListValidator { item_validator });

impl Validator for ListValidator {
    fn construct<'data>(
        &self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        state: &mut ConstructionState,
    ) -> ValResult<'data, PyObject> {
        if !state.recursive {
            return Ok(input.to_object(py));
        }

        // See if it's possible to treat input like a list
        let Ok(seq) = input.lax_list() else {
            // If not, it's impossible for us to deduce any nested types, so just return input as-is
            return Ok(input.to_object(py));
        };

        // Construct a vector of the (potentially) changed contents
        let output = match &self.item_validator {
            Some(ref item_validator) => seq.construct_to_vec(py, input, item_validator, state)?,
            None => seq.to_vec(py, input, "List", None)?,
        };

        // Ensure input type is output type (where possible)
        match seq {
            // In the cases below, copying is not really an option; so we instead return it as a concrete list
            GenericIterable::Iterator(_)
            | GenericIterable::DictKeys(_)
            | GenericIterable::DictValues(_)
            | GenericIterable::DictItems(_) => Ok(output.into_py(py)),
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
        let seq = input.validate_list(state.strict_or(self.strict))?;

        let output = match self.item_validator {
            Some(ref v) => seq.validate_to_vec(py, input, self.max_length, "List", v, state)?,
            None => match seq {
                GenericIterable::List(list) => {
                    length_check!(input, "List", self.min_length, self.max_length, list);
                    let list_copy = list.get_slice(0, usize::MAX);
                    return Ok(list_copy.into_py(py));
                }
                _ => seq.to_vec(py, input, "List", self.max_length)?,
            },
        };
        min_length_check!(input, "List", self.min_length, output);
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
