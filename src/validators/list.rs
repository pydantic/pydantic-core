use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::SchemaDict;
use crate::errors::{ErrorType, LocItem, ValError, ValResult};
use crate::input::Input;
use crate::input::{iterator, GenericIterable, JsonInput};
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
                    },
                    $input,
                ));
            }
        }
    }};
}
pub(crate) use length_check;

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

// Grouping of parameters that get passed around to reduce number of fn params
struct Extras<'s, 'data> {
    py: Python<'data>,
    extra: &'s Extra<'s>,
    definitions: &'data Definitions<CombinedValidator>,
    recursion_guard: &'s mut RecursionGuard,
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
        let strict = extra.strict.unwrap_or(self.strict);

        let length_constraints = iterator::LengthConstraints {
            min_length: self.min_length,
            max_length: self.max_length,
            max_input_length: None,
        };

        let mut extras: Extras = Extras {
            py,
            extra,
            definitions,
            recursion_guard,
        };

        let make_output = |capacity: usize| Ok(Vec::with_capacity(capacity));

        let mut output_func = |output: &mut Vec<PyObject>, ob: PyObject| -> ValResult<'data, usize> {
            output.push(ob);
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

        let generic_iterable = input
            .extract_iterable()
            .map_err(|_| ValError::new(ErrorType::ListType, input))?;
        let output = match (generic_iterable, strict) {
            // Always allow actual lists or JSON arrays
            (GenericIterable::JsonArray(iter), _) => iterator::validate_iterable(
                py,
                iter.iter().map(Ok),
                &mut json_validator_func,
                &mut output_func,
                length_constraints,
                FIELD_TYPE,
                input,
                &mut extras,
                make_output,
                Some(iter.len()),
                false,
            ),
            (GenericIterable::List(iter), _) => iterator::validate_iterable(
                py,
                iter.iter().map(Ok),
                &mut python_validator_func,
                &mut output_func,
                length_constraints,
                FIELD_TYPE,
                input,
                &mut extras,
                make_output,
                Some(iter.len()),
                false,
            ),
            // If not in strict mode we also accept any iterable except str/bytes
            (GenericIterable::String(_) | GenericIterable::Bytes(_), _) => {
                return Err(ValError::new(ErrorType::ListType, input))
            }
            (generic_iterable, false) => match generic_iterable.into_sequence_iterator(py) {
                Ok(iter) => {
                    let len = iter.size_hint().1;
                    iterator::validate_iterable(
                        py,
                        iter,
                        &mut python_validator_func,
                        &mut output_func,
                        length_constraints,
                        FIELD_TYPE,
                        input,
                        &mut extras,
                        make_output,
                        len,
                        false,
                    )
                }
                Err(_) => return Err(ValError::new(ErrorType::ListType, input)),
            },
            _ => return Err(ValError::new(ErrorType::ListType, input)),
        }?;

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
