use std::borrow::Cow;

use pyo3::exceptions::{PyKeyError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyInt, PyString};
use pyo3::{intern2, FromPyObject};

pub trait SchemaDict<'py> {
    fn get_as<T>(&self, key: &Py2<'_, PyString>) -> PyResult<Option<T>>
    where
        T: FromPyObject<'py>;

    fn get_as_req<T>(&self, key: &Py2<'_, PyString>) -> PyResult<T>
    where
        T: FromPyObject<'py>;
}

impl<'py> SchemaDict<'py> for Py2<'py, PyDict> {
    fn get_as<T>(&self, key: &Py2<'_, PyString>) -> PyResult<Option<T>>
    where
        T: FromPyObject<'py>,
    {
        match self.get_item(key)? {
            Some(t) => t.extract().map(Some),
            None => Ok(None),
        }
    }

    fn get_as_req<T>(&self, key: &Py2<'_, PyString>) -> PyResult<T>
    where
        T: FromPyObject<'py>,
    {
        match self.get_item(key)? {
            Some(t) => t.extract(),
            None => py_err!(PyKeyError; "{}", key),
        }
    }
}

impl<'py> SchemaDict<'py> for Option<&Py2<'py, PyDict>> {
    fn get_as<T>(&self, key: &Py2<'_, PyString>) -> PyResult<Option<T>>
    where
        T: FromPyObject<'py>,
    {
        match self {
            Some(d) => d.get_as(key),
            None => Ok(None),
        }
    }

    #[cfg_attr(has_coverage_attribute, coverage(off))]
    fn get_as_req<T>(&self, key: &Py2<'_, PyString>) -> PyResult<T>
    where
        T: FromPyObject<'py>,
    {
        match self {
            Some(d) => d.get_as_req(key),
            None => py_err!(PyKeyError; "{}", key),
        }
    }
}

macro_rules! py_error_type {
    ($error_type:ty; $msg:expr) => {
        <$error_type>::new_err($msg)
    };

    ($error_type:ty; $msg:expr, $( $msg_args:expr ),+ ) => {
        <$error_type>::new_err(format!($msg, $( $msg_args ),+))
    };
}
pub(crate) use py_error_type;

macro_rules! py_err {
    ($error_type:ty; $msg:expr) => {
        Err(crate::tools::py_error_type!($error_type; $msg))
    };

    ($error_type:ty; $msg:expr, $( $msg_args:expr ),+ ) => {
        Err(crate::tools::py_error_type!($error_type; $msg, $( $msg_args ),+))
    };
}
pub(crate) use py_err;

pub fn function_name(f: &Py2<'_, PyAny>) -> PyResult<String> {
    match f.getattr(intern2!(f.py(), "__name__")) {
        Ok(name) => name.extract(),
        _ => f.repr()?.extract(),
    }
}

pub fn safe_repr(v: &Py2<'_, PyAny>) -> Cow<'static, str> {
    if let Ok(s) = v.repr() {
        s.to_string_lossy().into_owned().into()
    } else if let Ok(name) = v.get_type().name() {
        format!("<unprintable {name} object>").into()
    } else {
        "<unprintable object>".into()
    }
}

pub fn extract_i64(v: &Py2<'_, PyAny>) -> PyResult<i64> {
    if v.is_instance_of::<PyInt>() {
        v.extract()
    } else {
        py_err!(PyTypeError; "expected int, got {}", safe_repr(v))
    }
}
