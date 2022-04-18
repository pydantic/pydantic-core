use serde_json::Value;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use super::shared::{int_as_bool, str_as_bool};
use super::traits::Input;
use crate::errors::{err_val_error, ErrorKind, ValResult};

#[derive(Debug)]
pub struct JsonInput(Value);

impl JsonInput {
    pub fn new(value: Value) -> JsonInput {
        JsonInput(value)
    }
}

impl ToPyObject for JsonInput {
    fn to_object(&self, py: Python) -> PyObject {
        value_to_py(&self.0, py)
    }
}

impl Input for JsonInput {
    fn validate_none(&self, py: Python) -> ValResult<()> {
        match &self.0 {
            Value::Null => Ok(()),
            _ => err_val_error!(py, self, kind = ErrorKind::NoneRequired),
        }
    }

    fn validate_str(&self, py: Python) -> ValResult<String> {
        match &self.0 {
            Value::String(s) => Ok(s.to_string()),
            Value::Number(n) => Ok(n.to_string()),
            _ => err_val_error!(py, self, kind = ErrorKind::StrType),
        }
    }

    fn validate_bool(&self, py: Python) -> ValResult<bool> {
        match &self.0 {
            Value::Bool(b) => Ok(*b),
            Value::String(s) => str_as_bool(py, s),
            Value::Number(n) => {
                if let Some(int) = n.as_i64() {
                    int_as_bool(py, int)
                } else {
                    err_val_error!(py, self, kind = ErrorKind::BoolParsing)
                }
            }
            _ => err_val_error!(py, self, kind = ErrorKind::BoolType),
        }
    }

    fn validate_int(&self, py: Python) -> ValResult<i64> {
        match &self.0 {
            Value::Number(n) => {
                if let Some(int) = n.as_i64() {
                    Ok(int)
                } else {
                    err_val_error!(py, self, kind = ErrorKind::IntFromFloat)
                }
            }
            Value::String(str) => match str.parse() {
                Ok(i) => Ok(i),
                Err(_) => err_val_error!(py, str, kind = ErrorKind::IntParsing),
            },
            _ => err_val_error!(py, self, kind = ErrorKind::IntType),
        }
    }

    fn validate_float(&self, py: Python) -> ValResult<f64> {
        match &self.0 {
            Value::Number(n) => {
                if let Some(float) = n.as_f64() {
                    Ok(float)
                } else {
                    err_val_error!(py, self, kind = ErrorKind::FloatParsing)
                }
            }
            Value::String(str) => match str.parse() {
                Ok(i) => Ok(i),
                Err(_) => err_val_error!(py, str, kind = ErrorKind::FloatParsing),
            },
            _ => err_val_error!(py, self, kind = ErrorKind::FloatType),
        }
    }

    fn validate_dict<'py>(&'py self, py: Python<'py>) -> ValResult<&'py PyDict> {
        match &self.0 {
            Value::Object(o) => {
                let dict = PyDict::new(py);
                for (k, v) in o.iter() {
                    let json_value = JsonInput(v.clone());
                    dict.set_item(k.into_py(py), json_value).unwrap();
                }
                Ok(dict)
            }
            _ => err_val_error!(py, self, kind = ErrorKind::DictType),
        }
    }

    fn validate_list<'py>(&'py self, py: Python<'py>) -> ValResult<&'py PyList> {
        match &self.0 {
            Value::Array(a) => {
                let items: Vec<PyObject> = a.iter().map(|v| value_to_py(v, py)).collect();
                Ok(PyList::new(py, items))
            }
            _ => err_val_error!(py, self, kind = ErrorKind::ListType),
        }
    }
}

fn value_to_py(value: &Value, py: Python) -> PyObject {
    match value {
        Value::Null => py.None(),
        Value::Bool(b) => b.into_py(py),
        Value::Number(n) => {
            if let Some(int) = n.as_i64() {
                int.into_py(py)
            } else if let Some(float) = n.as_f64() {
                float.into_py(py)
            } else {
                // TODO is this ok?
                0.into_py(py)
            }
        }
        Value::String(s) => s.to_string().into_py(py),
        Value::Array(a) => a.iter().map(|v| value_to_py(v, py)).collect::<Vec<_>>().into_py(py),
        Value::Object(o) => {
            let dict = PyDict::new(py);
            for (k, v) in o.iter() {
                dict.set_item(k, value_to_py(v, py)).unwrap();
            }
            dict.into_py(py)
        }
    }
}
