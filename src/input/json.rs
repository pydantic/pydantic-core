use serde_json::Value;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use super::shared::{int_as_bool, str_as_bool};
use super::traits::{Input, ListInput, ToPy};
use crate::errors::{err_val_error, ErrorKind, ValResult};

impl ToPy for Value {
    fn to_py(&self, py: Python) -> PyObject {
        match self {
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
            Value::Array(a) => a.iter().map(|v| v.to_py(py)).collect::<Vec<_>>().into_py(py),
            Value::Object(o) => {
                let dict = PyDict::new(py);
                for (k, v) in o.iter() {
                    dict.set_item(k, v.to_py(py)).unwrap();
                }
                dict.into_py(py)
            }
        }
    }
}

impl Input for Value {
    fn validate_none(&self, py: Python) -> ValResult<()> {
        match self {
            Value::Null => Ok(()),
            _ => err_val_error!(py, self, kind = ErrorKind::NoneRequired),
        }
    }

    fn validate_str(&self, py: Python) -> ValResult<String> {
        match self {
            Value::String(s) => Ok(s.to_string()),
            Value::Number(n) => Ok(n.to_string()),
            _ => err_val_error!(py, self, kind = ErrorKind::StrType),
        }
    }

    fn validate_bool(&self, py: Python) -> ValResult<bool> {
        match self {
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
        match self {
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
        match self {
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
        match self {
            Value::Object(o) => {
                let dict = PyDict::new(py);
                for (k, v) in o.iter() {
                    dict.set_item(k.to_py(py), v.to_py(py)).unwrap();
                }
                Ok(dict)
            }
            _ => err_val_error!(py, self, kind = ErrorKind::DictType),
        }
    }

    fn validate_list<'py>(&'py self, py: Python<'py>) -> ValResult<Box<dyn ListInput<'py> + 'py>> {
        match self {
            Value::Array(a) => Ok(Box::new(JsonListInput(a))),
            _ => err_val_error!(py, self, kind = ErrorKind::ListType),
        }
    }
}

struct JsonListInput<'py>(&'py Vec<Value>);

impl<'py> ToPy for JsonListInput<'py> {
    fn to_py(&self, py: Python) -> PyObject {
        self.0.iter().map(|v| v.to_py(py)).collect::<Vec<_>>().into_py(py)
    }
}

impl<'py> ListInput<'py> for JsonListInput<'py> {
    fn iter(&self) -> Box<dyn Iterator<Item = Box<&'py dyn Input>> + '_> {
        Box::new(self.0.iter().map(|item| Box::new(item as &'py dyn Input)))
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}
