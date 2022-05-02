use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};
use serde_json::{Map, Value};

use crate::errors::{err_val_error, ErrorKind, InputValue, LocItem, ValResult};

use super::shared::{float_as_int, int_as_bool, str_as_bool, str_as_int};
use super::traits::{DictInput, Input, ListInput, ToLocItem};
use super::parse_json::{JsonInput, JsonArray, JsonObject};

impl Input for JsonInput {
    fn is_none(&self) -> bool {
        matches!(self, Value::Null)
    }

    fn strict_str(&self) -> ValResult<String> {
        match self {
            Value::String(s) => Ok(s.to_string()),
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::StrType),
        }
    }

    fn lax_str(&self) -> ValResult<String> {
        match self {
            Value::String(s) => Ok(s.to_string()),
            Value::Number(n) => Ok(n.to_string()),
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::StrType),
        }
    }

    fn strict_bool(&self) -> ValResult<bool> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::BoolType),
        }
    }

    fn lax_bool(&self) -> ValResult<bool> {
        match self {
            Value::Bool(b) => Ok(*b),
            Value::String(s) => str_as_bool(self, s),
            Value::Number(n) => {
                if let Some(int) = n.as_i64() {
                    int_as_bool(self, int)
                } else {
                    err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::BoolParsing)
                }
            }
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::BoolType),
        }
    }

    fn strict_int(&self) -> ValResult<i64> {
        match self {
            Value::Number(n) => {
                if let Some(int) = n.as_i64() {
                    Ok(int)
                } else {
                    err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::IntType)
                }
            }
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::IntType),
        }
    }

    fn lax_int(&self) -> ValResult<i64> {
        match self {
            Value::Bool(b) => match *b {
                true => Ok(1),
                false => Ok(0),
            },
            Value::Number(n) => {
                if let Some(int) = n.as_i64() {
                    Ok(int)
                } else if let Some(float) = n.as_f64() {
                    float_as_int(self, float)
                } else {
                    err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::IntType)
                }
            }
            Value::String(str) => str_as_int(self, str),
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::IntType),
        }
    }

    fn strict_float(&self) -> ValResult<f64> {
        match self {
            Value::Number(n) => {
                if let Some(float) = n.as_f64() {
                    Ok(float)
                } else {
                    err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::FloatParsing)
                }
            }
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::FloatType),
        }
    }

    fn lax_float(&self) -> ValResult<f64> {
        match self {
            Value::Bool(b) => match *b {
                true => Ok(1.0),
                false => Ok(0.0),
            },
            Value::Number(n) => {
                if let Some(float) = n.as_f64() {
                    Ok(float)
                } else {
                    err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::FloatParsing)
                }
            }
            Value::String(str) => match str.parse() {
                Ok(i) => Ok(i),
                Err(_) => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::FloatParsing),
            },
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::FloatType),
        }
    }

    fn strict_model_check(&self, _class: &PyType) -> ValResult<bool> {
        Ok(false)
    }

    fn strict_dict<'data>(&'data self) -> ValResult<Box<dyn DictInput<'data> + 'data>> {
        match self {
            Value::Object(dict) => Ok(Box::new(dict)),
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::DictType),
        }
    }

    fn strict_list<'data>(&'data self) -> ValResult<Box<dyn ListInput<'data> + 'data>> {
        match self {
            Value::Array(a) => Ok(Box::new(a)),
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::ListType),
        }
    }

    fn strict_set<'data>(&'data self) -> ValResult<Box<dyn ListInput<'data> + 'data>> {
        // we allow a list here since otherwise it would be impossible to create a set from JSON
        match self {
            Value::Array(a) => Ok(Box::new(a)),
            _ => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::SetType),
        }
    }
}

/// Required for Dict keys so the string can behave like an Input
impl Input for String {
    fn is_none(&self) -> bool {
        false
    }

    fn strict_str(&self) -> ValResult<String> {
        Ok(self.clone())
    }

    fn lax_str(&self) -> ValResult<String> {
        Ok(self.clone())
    }

    fn strict_bool(&self) -> ValResult<bool> {
        err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::BoolType)
    }

    fn lax_bool(&self) -> ValResult<bool> {
        str_as_bool(self, self)
    }

    fn strict_int(&self) -> ValResult<i64> {
        err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::IntType)
    }

    fn lax_int(&self) -> ValResult<i64> {
        match self.parse() {
            Ok(i) => Ok(i),
            Err(_) => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::IntParsing),
        }
    }

    fn strict_float(&self) -> ValResult<f64> {
        err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::FloatType)
    }

    fn lax_float(&self) -> ValResult<f64> {
        match self.parse() {
            Ok(i) => Ok(i),
            Err(_) => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::FloatParsing),
        }
    }

    fn strict_model_check(&self, _class: &PyType) -> ValResult<bool> {
        Ok(false)
    }

    fn strict_dict<'data>(&'data self) -> ValResult<Box<dyn DictInput<'data> + 'data>> {
        err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::DictType)
    }

    fn strict_list<'data>(&'data self) -> ValResult<Box<dyn ListInput<'data> + 'data>> {
        err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::ListType)
    }

    fn strict_set<'data>(&'data self) -> ValResult<Box<dyn ListInput<'data> + 'data>> {
        err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::SetType)
    }
}
