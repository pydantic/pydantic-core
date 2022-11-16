use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyMapping, PyTuple};

use crate::build_tools::{is_strict, SchemaDict};
use crate::errors::{py_err_string, ErrorType, ValError, ValLineError, ValResult};
use crate::input::{GenericMapping, Input, JsonObject};
use crate::recursion_guard::RecursionGuard;

use super::any::AnyValidator;
use super::list::length_check;
use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct DictValidator {
    strict: bool,
    key_validator: Box<CombinedValidator>,
    value_validator: Box<CombinedValidator>,
    min_length: Option<usize>,
    max_length: Option<usize>,
    name: String,
}

impl BuildValidator for DictValidator {
    const EXPECTED_TYPE: &'static str = "dict";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let key_validator = match schema.get_item(intern!(py, "keys_schema")) {
            Some(schema) => Box::new(build_validator(schema, config, build_context)?),
            None => Box::new(AnyValidator::build(schema, config, build_context)?),
        };
        let value_validator = match schema.get_item(intern!(py, "values_schema")) {
            Some(d) => Box::new(build_validator(d, config, build_context)?),
            None => Box::new(AnyValidator::build(schema, config, build_context)?),
        };
        let name = format!(
            "{}[{},{}]",
            Self::EXPECTED_TYPE,
            key_validator.get_name(),
            value_validator.get_name()
        );
        Ok(Self {
            strict: is_strict(schema, config)?,
            key_validator,
            value_validator,
            min_length: schema.get_as(intern!(py, "min_length"))?,
            max_length: schema.get_as(intern!(py, "max_length"))?,
            name,
        }
        .into())
    }
}

impl Validator for DictValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let dict = input.validate_dict(extra.strict.unwrap_or(self.strict))?;
        match dict {
            GenericMapping::PyDict(py_dict) => self.validate_dict(py, input, py_dict, extra, slots, recursion_guard),
            GenericMapping::PyMapping(mapping) => {
                self.validate_mapping(py, input, mapping, extra, slots, recursion_guard)
            }
            GenericMapping::PyGetAttr(_) => unreachable!(),
            GenericMapping::JsonObject(json_object) => {
                self.validate_json_object(py, input, json_object, extra, slots, recursion_guard)
            }
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn complete(&mut self, build_context: &BuildContext) -> PyResult<()> {
        self.key_validator.complete(build_context)?;
        self.value_validator.complete(build_context)
    }
}

macro_rules! build_validate {
    ($name:ident, $dict_type:ty, $iter_macro:ident, $extract_macro:ident) => {
        fn $name<'s, 'data>(
            &'s self,
            py: Python<'data>,
            input: &'data impl Input<'data>,
            dict: &'data $dict_type,
            extra: &Extra,
            slots: &'data [CombinedValidator],
            recursion_guard: &'s mut RecursionGuard,
        ) -> ValResult<'data, PyObject> {
            let output = PyDict::new(py);
            let mut errors: Vec<ValLineError> = Vec::new();

            let key_validator = self.key_validator.as_ref();
            let value_validator = self.value_validator.as_ref();
            for elem in $iter_macro!(py, dict, input) {
                let (key, value) = $extract_macro!(py, elem, input);
                let output_key = match key_validator.validate(py, key, extra, slots, recursion_guard) {
                    Ok(value) => Some(value),
                    Err(ValError::LineErrors(line_errors)) => {
                        for err in line_errors {
                            // these are added in reverse order so [key] is shunted along by the second call
                            errors.push(
                                err.with_outer_location("[key]".into())
                                    .with_outer_location(key.as_loc_item()),
                            );
                        }
                        None
                    }
                    Err(ValError::Omit) => continue,
                    Err(err) => return Err(err),
                };
                let output_value = match value_validator.validate(py, value, extra, slots, recursion_guard) {
                    Ok(value) => Some(value),
                    Err(ValError::LineErrors(line_errors)) => {
                        for err in line_errors {
                            errors.push(err.with_outer_location(key.as_loc_item()));
                        }
                        None
                    }
                    Err(ValError::Omit) => continue,
                    Err(err) => return Err(err),
                };
                if let (Some(key), Some(value)) = (output_key, output_value) {
                    output.set_item(key, value)?;
                }
            }

            if errors.is_empty() {
                length_check!(input, "Dictionary", self.min_length, self.max_length, output);
                Ok(output.into())
            } else {
                Err(ValError::LineErrors(errors))
            }
        }
    };
}

macro_rules! iter_simple {
    ($py:ident, $dict:ident, $input:ident) => {
        $dict.iter()
    };
}

fn mapping_err<'py>(err: PyErr, py: Python<'py>, input: &'py impl Input<'py>) -> ValError<'py> {
    ValError::new(
        ErrorType::MappingType {
            error: py_err_string(py, err).into(),
        },
        input,
    )
}

macro_rules! iter_mapping {
    ($py:ident, $mapping:ident, $input:ident) => {
        $mapping
            .items()
            .map_err(|e| mapping_err(e, $py, $input))?
            .iter()
            .map_err(|e| mapping_err(e, $py, $input))?
    };
}

macro_rules! extract_kv_simple {
    ($py:ident, $item:ident, $input:ident) => {
        $item
    };
}

macro_rules! extract_kv_mapping {
    ($py:ident, $item:ident, $input:ident) => {
        extract_mapping_item($py, $item, $input)?
    };
}

const MAPPING_TUPLE_ERROR: &str = "Mapping items must be tuples of (key, value) pairs";

fn extract_mapping_item<'py>(
    py: Python<'py>,
    result: PyResult<&'py PyAny>,
    input: &'py impl Input<'py>,
) -> ValResult<'py, (&'py PyAny, &'py PyAny)> {
    let item = result.map_err(|e| mapping_err(e, py, input))?;
    let tuple: &PyTuple = item.cast_as().map_err(|_| {
        ValError::new(
            ErrorType::MappingType {
                error: MAPPING_TUPLE_ERROR.into(),
            },
            input,
        )
    })?;
    if tuple.len() != 2 {
        return Err(ValError::new(
            ErrorType::MappingType {
                error: MAPPING_TUPLE_ERROR.into(),
            },
            input,
        ));
    };
    #[cfg(PyPy)]
    let key = tuple.get_item(0)?;
    #[cfg(PyPy)]
    let value = tuple.get_item(1)?;
    #[cfg(not(PyPy))]
    let key = unsafe { tuple.get_item_unchecked(0) };
    #[cfg(not(PyPy))]
    let value = unsafe { tuple.get_item_unchecked(1) };
    Ok((key, value))
}

// macro_rules! iter_mapping {
//     ($obj:ident) => {
//         $obj.iter_mapping()?
//     };
// }
//
// macro_rules! extract_dict_elem {
//     ($elem:ident, $input:ident) => {
//         $elem
//     };
// }
//
// macro_rules! extract_mapping_elem {
//     ($elem:ident, $input:ident) => {
//         extract_mapping_elem($elem, $input)?
//     };
// }
//
// trait IterMapping {
//     fn iter_mapping<'data>(&'data self, input: &'data impl Input<'data>) -> ValResult<'data, &PyIterator>;
// }
//
// impl IterMapping for PyMapping {
//     fn iter_mapping<'data>(&'data self, input: &'data impl Input<'data>) -> ValResult<'data, &PyIterator> {
//         let items = match self.items() {
//             Ok(items) => items,
//             Err(err) => return Err(ValError::new(ErrorType::MappingType { error: err.to_string() }, input)),
//         };
//         return Ok(items.iter()?);
//     }
// }

impl DictValidator {
    build_validate!(validate_dict, PyDict, iter_simple, extract_kv_simple);
    build_validate!(validate_mapping, PyMapping, iter_mapping, extract_kv_mapping);
    build_validate!(validate_json_object, JsonObject, iter_simple, extract_kv_simple);
}
