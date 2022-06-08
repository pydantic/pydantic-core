use std::str::from_utf8;

use pyo3::prelude::*;
use pyo3::types::{
    PyBytes, PyDate, PyDateTime, PyDict, PyFrozenSet, PyInt, PyList, PyMapping, PySet, PyString, PyTuple, PyType,
};

use crate::errors::{as_internal, err_val_error, ErrorKind, InputValue, ValResult};
use crate::input::shared::bytes_as_datetime;

use super::generics::{GenericMapping, GenericSequence};
use super::input_abstract::Input;
use super::shared::{
    bytes_as_date, date_as_py_date, date_from_datetime, datetime_as_py_datetime, float_as_datetime, float_as_int,
    int_as_bool, int_as_datetime, str_as_bool, str_as_int,
};

impl Input for PyAny {
    fn is_none(&self) -> bool {
        self.is_none()
    }

    fn strict_str(&self) -> ValResult<String> {
        if let Ok(py_str) = self.cast_as::<PyString>() {
            py_str.extract().map_err(as_internal)
        } else {
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::StrType)
        }
    }

    fn lax_str(&self) -> ValResult<String> {
        if let Ok(py_str) = self.cast_as::<PyString>() {
            py_str.extract().map_err(as_internal)
        } else if let Ok(bytes) = self.cast_as::<PyBytes>() {
            let str = match from_utf8(bytes.as_bytes()) {
                Ok(s) => s.to_string(),
                Err(_) => {
                    return err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::StrUnicode)
                }
            };
            Ok(str)
        } else if self.extract::<bool>().is_ok() {
            // do this before int and float parsing as `False` is cast to `0` and we don't want False to
            // be returned as a string
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::StrType)
        } else if let Ok(int) = self.cast_as::<PyInt>() {
            let int = i64::extract(int).map_err(as_internal)?;
            Ok(int.to_string())
        } else if let Ok(float) = f64::extract(self) {
            // don't cast_as here so Decimals are covered - internally f64:extract uses PyFloat_AsDouble
            Ok(float.to_string())
        } else {
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::StrType)
        }
    }

    fn strict_bool(&self) -> ValResult<bool> {
        if let Ok(bool) = self.extract::<bool>() {
            Ok(bool)
        } else {
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::BoolType)
        }
    }

    fn lax_bool(&self) -> ValResult<bool> {
        if let Ok(bool) = self.extract::<bool>() {
            Ok(bool)
        } else if let Some(str) = _maybe_as_string(self, ErrorKind::BoolParsing)? {
            str_as_bool(self, &str)
        } else if let Ok(int) = self.extract::<i64>() {
            int_as_bool(self, int)
        } else {
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::BoolType)
        }
    }

    fn strict_int(&self) -> ValResult<i64> {
        // bool check has to come before int check as bools would be cast to ints below
        if self.extract::<bool>().is_ok() {
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::IntType)
        } else if let Ok(int) = self.extract::<i64>() {
            Ok(int)
        } else {
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::IntType)
        }
    }

    fn lax_int(&self) -> ValResult<i64> {
        if let Ok(int) = self.extract::<i64>() {
            Ok(int)
        } else if let Some(str) = _maybe_as_string(self, ErrorKind::IntParsing)? {
            str_as_int(self, &str)
        } else if let Ok(float) = self.lax_float() {
            float_as_int(self, float)
        } else {
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::IntType)
        }
    }

    fn strict_float(&self) -> ValResult<f64> {
        if self.extract::<bool>().is_ok() {
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::FloatType)
        } else if let Ok(float) = self.extract::<f64>() {
            Ok(float)
        } else {
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::FloatType)
        }
    }

    fn lax_float(&self) -> ValResult<f64> {
        if let Ok(float) = self.extract::<f64>() {
            Ok(float)
        } else if let Some(str) = _maybe_as_string(self, ErrorKind::FloatParsing)? {
            match str.parse() {
                Ok(i) => Ok(i),
                Err(_) => err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::FloatParsing),
            }
        } else {
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::FloatType)
        }
    }

    fn strict_model_check(&self, class: &PyType) -> ValResult<bool> {
        self.get_type().eq(class).map_err(as_internal)
    }

    fn strict_dict<'data>(&'data self) -> ValResult<GenericMapping<'data>> {
        if let Ok(dict) = self.cast_as::<PyDict>() {
            Ok(dict.into())
        } else {
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::DictType)
        }
    }

    fn lax_dict<'data>(&'data self, try_instance: bool) -> ValResult<GenericMapping<'data>> {
        if let Ok(dict) = self.cast_as::<PyDict>() {
            Ok(dict.into())
        } else if let Ok(mapping) = self.cast_as::<PyMapping>() {
            // this is ugly, but we'd have to do it in `input_iter` anyway
            // we could perhaps use an indexmap instead of a python dict?
            let dict = match mapping_as_dict(mapping) {
                Ok(dict) => dict,
                Err(err) => {
                    return err_val_error!(
                        input_value = InputValue::InputRef(self),
                        message = Some(err.to_string()),
                        kind = ErrorKind::DictFromMapping
                    )
                }
            };
            Ok(dict.into())
        } else if try_instance {
            let inner_dict = match instance_as_dict(self) {
                Ok(dict) => dict,
                Err(err) => {
                    return err_val_error!(
                        input_value = InputValue::InputRef(self),
                        message = Some(err.to_string()),
                        kind = ErrorKind::DictFromObject
                    )
                }
            };
            inner_dict.lax_dict(false)
        } else {
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::DictType)
        }
    }

    fn strict_list<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        if let Ok(list) = self.cast_as::<PyList>() {
            Ok(list.into())
        } else {
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::ListType)
        }
    }

    fn lax_list<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        if let Ok(list) = self.cast_as::<PyList>() {
            Ok(list.into())
        } else if let Ok(tuple) = self.cast_as::<PyTuple>() {
            Ok(tuple.into())
        } else if let Ok(set) = self.cast_as::<PySet>() {
            Ok(set.into())
        } else if let Ok(frozen_set) = self.cast_as::<PyFrozenSet>() {
            Ok(frozen_set.into())
        } else {
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::ListType)
        }
    }

    fn strict_set<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        if let Ok(set) = self.cast_as::<PySet>() {
            Ok(set.into())
        } else {
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::SetType)
        }
    }

    fn lax_set<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        if let Ok(set) = self.cast_as::<PySet>() {
            Ok(set.into())
        } else if let Ok(list) = self.cast_as::<PyList>() {
            Ok(list.into())
        } else if let Ok(tuple) = self.cast_as::<PyTuple>() {
            Ok(tuple.into())
        } else if let Ok(frozen_set) = self.cast_as::<PyFrozenSet>() {
            Ok(frozen_set.into())
        } else {
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::SetType)
        }
    }

    fn strict_date<'data>(&'data self, _py: Python<'data>) -> ValResult<&'data PyDate> {
        if let Ok(date) = self.cast_as::<PyDate>() {
            Ok(date)
        } else {
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::DateType)
        }
    }

    fn lax_date<'data>(&'data self, py: Python<'data>) -> ValResult<&'data PyDate> {
        if let Ok(date) = self.cast_as::<PyDate>() {
            return Ok(date);
        }

        let parse_date = || {
            if let Ok(str) = self.extract::<String>() {
                let date = bytes_as_date(self, str.as_bytes())?;
                date_as_py_date!(py, date)
            } else if let Ok(py_bytes) = self.cast_as::<PyBytes>() {
                let date = bytes_as_date(self, py_bytes.as_bytes())?;
                date_as_py_date!(py, date)
            } else {
                err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::DateType)
            }
        };

        match parse_date() {
            Ok(date) => Ok(date),
            Err(err) => match self.lax_datetime(py) {
                Ok(dt) => date_from_datetime(self, py, dt),
                _ => Err(err),
            },
        }
    }

    fn strict_datetime<'data>(&'data self, _py: Python<'data>) -> ValResult<&'data PyDateTime> {
        if let Ok(dt) = self.cast_as::<PyDateTime>() {
            Ok(dt)
        } else {
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::DateTimeType)
        }
    }

    fn lax_datetime<'data>(&'data self, py: Python<'data>) -> ValResult<&'data PyDateTime> {
        if let Ok(dt) = self.cast_as::<PyDateTime>() {
            Ok(dt)
        } else if let Ok(str) = self.extract::<String>() {
            let dt = bytes_as_datetime(self, str.as_bytes())?;
            datetime_as_py_datetime!(py, dt)
        } else if let Ok(py_bytes) = self.cast_as::<PyBytes>() {
            let dt = bytes_as_datetime(self, py_bytes.as_bytes())?;
            datetime_as_py_datetime!(py, dt)
        } else if let Ok(int) = self.extract::<i64>() {
            let dt = int_as_datetime(self, int, 0)?;
            datetime_as_py_datetime!(py, dt)
        } else if let Ok(float) = self.extract::<f64>() {
            let dt = float_as_datetime(self, float)?;
            datetime_as_py_datetime!(py, dt)
        } else {
            err_val_error!(input_value = InputValue::InputRef(self), kind = ErrorKind::DateTimeType)
        }
    }
}

fn mapping_as_dict(mapping: &PyMapping) -> PyResult<&PyDict> {
    let seq = mapping.items()?;
    let dict = PyDict::new(mapping.py());
    for r in seq.iter()? {
        let t: &PyTuple = r?.extract()?;
        let k = t.get_item(0)?;
        let v = t.get_item(1)?;
        dict.set_item(k, v)?;
    }
    Ok(dict)
}

/// This is equivalent to `GetterDict` in pydantic v1
fn instance_as_dict(instance: &PyAny) -> PyResult<&PyDict> {
    let dict = PyDict::new(instance.py());
    for k_any in instance.dir() {
        let k_str: &str = k_any.extract()?;
        if !k_str.starts_with('_') {
            let v = instance.getattr(k_any)?;
            dict.set_item(k_any, v)?;
        }
    }
    Ok(dict)
}

/// Utility for extracting a string from a PyAny, if possible.
fn _maybe_as_string(v: &PyAny, unicode_error: ErrorKind) -> ValResult<Option<String>> {
    if let Ok(str) = v.extract::<String>() {
        Ok(Some(str))
    } else if let Ok(bytes) = v.cast_as::<PyBytes>() {
        let str = match from_utf8(bytes.as_bytes()) {
            Ok(s) => s.to_string(),
            Err(_) => return err_val_error!(input_value = InputValue::InputRef(v), kind = unicode_error),
        };
        Ok(Some(str))
    } else {
        Ok(None)
    }
}
