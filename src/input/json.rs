use serde_json::Value;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde_json::Map;

use super::shared::{int_as_bool, str_as_bool};
use super::traits::{DictInput, Input, ListInput, ToLocItem, ToPy};
use crate::errors::{as_internal, err_val_error, ErrorKind, LocItem, ValResult};
use crate::utils::py_error;

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
                    panic!("{:?} is not a valid number", n)
                }
            }
            Value::String(s) => s.clone().into_py(py),
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

impl ToLocItem for Value {
    fn to_loc(&self) -> ValResult<LocItem> {
        match self {
            Value::Number(n) => {
                if let Some(int) = n.as_i64() {
                    Ok(LocItem::I(int as usize))
                } else if let Some(float) = n.as_f64() {
                    Ok(LocItem::I(float as usize))
                } else {
                    py_error!(PyValueError; "{:?} is not a valid number", n).map_err(as_internal)
                }
            }
            Value::String(s) => Ok(LocItem::S(s.to_string())),
            v => Ok(LocItem::S(format!("{:?}", v))),
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
        dbg!(self);
        match self {
            Value::Number(n) => {
                if let Some(int) = n.as_i64() {
                    Ok(int)
                } else if let Some(float) = n.as_f64() {
                    if float % 1.0 == 0.0 {
                        Ok(float as i64)
                    } else {
                        err_val_error!(py, float, kind = ErrorKind::IntFromFloat)
                    }
                } else {
                    err_val_error!(py, self, kind = ErrorKind::IntType)
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

    fn validate_dict<'py>(&'py self, py: Python<'py>) -> ValResult<Box<dyn DictInput<'py> + 'py>> {
        match self {
            Value::Object(dict) => Ok(Box::new(JsonDictInput(dict))),
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

struct JsonDictInput<'py>(&'py Map<String, Value>);

impl<'py> ToPy for JsonDictInput<'py> {
    fn to_py(&self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        for (k, v) in self.0.iter() {
            dict.set_item(k, v.to_py(py)).unwrap();
        }
        dict.into_py(py)
    }
}

impl<'py> DictInput<'py> for JsonDictInput<'py> {
    fn iter(&self) -> Box<dyn Iterator<Item = (&dyn Input, &dyn Input)> + '_> {
        Box::new(self.0.iter().map(|(k, v)| (k as &dyn Input, v as &dyn Input)))
    }

    fn get_item(&self, key: &str) -> Option<&dyn Input> {
        self.0.get(key).map(|item| item as &dyn Input)
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

struct JsonListInput<'py>(&'py Vec<Value>);

impl<'py> ToPy for JsonListInput<'py> {
    fn to_py(&self, py: Python) -> PyObject {
        self.0.iter().map(|v| v.to_py(py)).collect::<Vec<_>>().into_py(py)
    }
}

impl<'py> ListInput<'py> for JsonListInput<'py> {
    // this is ugly, is there any way to avoid the map, one of the boxes and/or avoid the duplication?
    fn iter(&self) -> Box<dyn Iterator<Item = &dyn Input> + '_> {
        Box::new(self.0.iter().map(|item| item as &dyn Input))
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

/// Required for Dict keys so the string can behave like an Input
impl Input for String {
    fn validate_none(&self, py: Python) -> ValResult<()> {
        err_val_error!(py, self, kind = ErrorKind::NoneRequired)
    }

    fn validate_str(&self, _py: Python) -> ValResult<String> {
        Ok(self.clone())
    }

    fn validate_bool(&self, py: Python) -> ValResult<bool> {
        str_as_bool(py, self)
    }

    fn validate_int(&self, py: Python) -> ValResult<i64> {
        match self.parse() {
            Ok(i) => Ok(i),
            Err(_) => err_val_error!(py, self, kind = ErrorKind::IntParsing),
        }
    }

    fn validate_float(&self, py: Python) -> ValResult<f64> {
        match self.parse() {
            Ok(i) => Ok(i),
            Err(_) => err_val_error!(py, self, kind = ErrorKind::FloatParsing),
        }
    }

    fn validate_dict<'py>(&'py self, py: Python<'py>) -> ValResult<Box<dyn DictInput<'py> + 'py>> {
        err_val_error!(py, self, kind = ErrorKind::DictType)
    }

    fn validate_list<'py>(&'py self, py: Python<'py>) -> ValResult<Box<dyn ListInput<'py> + 'py>> {
        err_val_error!(py, self, kind = ErrorKind::ListType)
    }
}
