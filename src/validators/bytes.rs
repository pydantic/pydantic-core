use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

use crate::build_tools::{is_strict, schema_or_config};
use crate::errors::{context, err_val_error, ErrorKind, InputValue, ValResult};
use crate::input::Input;

use super::{BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct BytesValidator;

impl BuildValidator for BytesValidator {
    const EXPECTED_TYPE: &'static str = "bytes";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        _build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let use_constrained = schema.get_item("max_length").is_some()
            || schema.get_item("min_length").is_some()
            || match config {
                Some(config) => {
                    config.get_item("bytes_max_length").is_some() || config.get_item("bytes_min_length").is_some()
                }
                None => false,
            };
        if use_constrained {
            BytesConstrainedValidator::build(schema, config)
        } else if is_strict(schema, config)? {
            StrictBytesValidator::build()
        } else {
            Ok(Self.into())
        }
    }
}

impl Validator for BytesValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        _extra: &Extra,
        _slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        Ok(PyBytes::new(py, &input.lax_bytes()?).into_py(py))
    }

    fn validate_strict<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        _extra: &Extra,
        _slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        Ok(PyBytes::new(py, &input.strict_bytes()?).into_py(py))
    }

    fn get_name(&self, _py: Python) -> String {
        Self::EXPECTED_TYPE.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct StrictBytesValidator;

impl StrictBytesValidator {
    fn build() -> PyResult<CombinedValidator> {
        Ok(Self.into())
    }
}

impl Validator for StrictBytesValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        _extra: &Extra,
        _slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        Ok(PyBytes::new(py, &input.strict_bytes()?).into_py(py))
    }

    fn get_name(&self, _py: Python) -> String {
        "strict-bytes".to_string()
    }
}

#[derive(Debug, Clone)]
pub struct BytesConstrainedValidator {
    strict: bool,
    max_length: Option<usize>,
    min_length: Option<usize>,
}

impl Validator for BytesConstrainedValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        _extra: &Extra,
        _slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        let bytes = match self.strict {
            true => input.strict_bytes()?,
            false => input.lax_bytes()?,
        };
        self._validation_logic(py, input, bytes)
    }

    fn validate_strict<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        _extra: &Extra,
        _slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        self._validation_logic(py, input, input.strict_bytes()?)
    }

    fn get_name(&self, _py: Python) -> String {
        "constrained-bytes".to_string()
    }
}

impl BytesConstrainedValidator {
    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<CombinedValidator> {
        let min_length: Option<usize> = schema_or_config(schema, config, "min_length", "bytes_min_length")?;
        let max_length: Option<usize> = schema_or_config(schema, config, "max_length", "bytes_max_length")?;

        Ok(Self {
            strict: is_strict(schema, config)?,
            min_length,
            max_length,
        }
        .into())
    }

    fn _validation_logic<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        bytes: Vec<u8>,
    ) -> ValResult<'data, PyObject> {
        let bytes = bytes;
        if let Some(min_length) = self.min_length {
            if bytes.len() < min_length {
                return err_val_error!(
                    input_value = InputValue::InputRef(input),
                    kind = ErrorKind::BytesTooShort,
                    context = context!("min_length" => min_length)
                );
            }
        }
        if let Some(max_length) = self.max_length {
            if bytes.len() > max_length {
                return err_val_error!(
                    input_value = InputValue::InputRef(input),
                    kind = ErrorKind::BytesTooLong,
                    context = context!("max_length" => max_length)
                );
            }
        }

        let py_bytes = PyBytes::new(py, &bytes);
        Ok(py_bytes.into_py(py))
    }
}
