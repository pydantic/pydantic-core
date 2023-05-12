use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::SchemaDict;
use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::Input;
use crate::input::{iterator, AnyIterable, JsonInput};
use crate::recursion_guard::RecursionGuard;

use super::{build_validator, BuildValidator, CombinedValidator, Definitions, DefinitionsBuilder, Extra, Validator};

#[derive(Debug, Clone)]
pub struct ListValidator {
    strict: bool,
    allow_any_iter: bool,
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
            allow_any_iter: schema.get_as(pyo3::intern!(py, "allow_any_iter"))?.unwrap_or(false),
            item_validator,
            min_length: schema.get_as(pyo3::intern!(py, "min_length"))?,
            max_length: schema.get_as(pyo3::intern!(py, "max_length"))?,
            name,
        }
        .into())
    }
}

impl Validator for ListValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        definitions: &'data Definitions<CombinedValidator>,
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let strict = extra.strict.unwrap_or(self.strict);

        macro_rules! validate_python {
            ($iter:expr, $len:expr) => {{
                let init_capacity = iterator::calculate_output_init_capacity($len, self.max_length);
                let mut output = Vec::with_capacity(init_capacity);

                let mut validation_func = |ob: &'data PyAny| -> ValResult<'data, _> {
                    match &self.item_validator {
                        Some(v) => v.validate(py, ob, extra, definitions, recursion_guard),
                        None => Ok(ob.into_py(py)),
                    }
                };

                let mut output_func = |ob: PyObject| -> ValResult<'data, usize> {
                    output.push(ob);
                    Ok(output.len())
                };
                iterator::validate_with_errors(
                    py,
                    $iter,
                    &mut validation_func,
                    &mut output_func,
                    self.min_length,
                    self.max_length,
                    "List",
                    input,
                )?;

                output
            }};
        }

        macro_rules! validate_any_iter {
            ($iter:expr, $len:expr) => {{
                if self.allow_any_iter {
                    validate_python!($iter, $len)
                } else {
                    return Err(ValError::new(ErrorType::ListType, input));
                }
            }};
        }

        // We're a bit inconsistent with what we accept as inputs or don't
        // E.g. sets are not a sequence but we accept them
        let map_default_err = |_| {
            if self.allow_any_iter {
                ValError::new(ErrorType::IterableType, input)
            } else {
                ValError::new(ErrorType::ListType, input)
            }
        };
        let output = match (input.extract_iterable().map_err(map_default_err)?, strict) {
            (AnyIterable::List(iter), _) => validate_python!(iter.iter().map(Ok), Some(iter.len())),
            (AnyIterable::JsonArray(iter), _) => {
                let init_capacity = iterator::calculate_output_init_capacity(Some(iter.len()), self.max_length);
                let mut output = Vec::with_capacity(init_capacity);

                let mut validation_func = |ob: &'data JsonInput| -> ValResult<'data, PyObject> {
                    match &self.item_validator {
                        Some(v) => v.validate(py, ob, extra, definitions, recursion_guard),
                        None => Ok(ob.to_object(py)),
                    }
                };

                let mut output_func = |ob: PyObject| -> ValResult<'data, usize> {
                    output.push(ob);
                    Ok(output.len())
                };
                iterator::validate_with_errors(
                    py,
                    iter.iter().map(Ok),
                    &mut validation_func,
                    &mut output_func,
                    self.min_length,
                    self.max_length,
                    "List",
                    input,
                )?;
                output
            }
            (AnyIterable::Tuple(iter), false) => validate_python!(iter.iter().map(Ok), Some(iter.len())),
            (AnyIterable::Set(iter), false) => validate_any_iter!(iter.iter().map(Ok), Some(iter.len())),
            (AnyIterable::FrozenSet(iter), false) => validate_any_iter!(iter.iter().map(Ok), Some(iter.len())),
            #[cfg(not(PyPy))]
            (AnyIterable::DictKeys(iter), false) => validate_python!(iter.iter()?, iter.len().ok()),
            #[cfg(not(PyPy))]
            (AnyIterable::DictValues(iter), false) => validate_python!(iter.iter()?, iter.len().ok()),
            #[cfg(not(PyPy))]
            (AnyIterable::DictItems(iter), false) => validate_python!(iter.iter()?, iter.len().ok()),
            (AnyIterable::Sequence(iter), false) => validate_python!(iter.iter()?, iter.len().ok()),
            (AnyIterable::Iterator(iter), false) => validate_any_iter!(iter.iter()?, iter.len().ok()),
            (AnyIterable::Mapping(iter), false) => validate_any_iter!(iter.iter()?, iter.len().ok()),
            (AnyIterable::Dict(iter), false) => validate_any_iter!(iter.as_ref().iter()?, Some(iter.len())),
            (_, _) => return Err(ValError::new(ErrorType::ListType, input)),
        };

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
