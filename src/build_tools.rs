use std::borrow::Cow;
use std::error::Error;
use std::fmt;

use pyo3::exceptions::{PyException, PyKeyError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString};
use pyo3::{intern, FromPyObject, PyErrArguments};

use crate::errors::ValError;
use crate::ValidationError;

pub trait SchemaDict<'py> {
    fn get_as<T>(&'py self, key: &PyString) -> PyResult<Option<T>>
    where
        T: FromPyObject<'py>;

    fn get_as_req<T>(&'py self, key: &PyString) -> PyResult<T>
    where
        T: FromPyObject<'py>;
}

impl<'py> SchemaDict<'py> for PyDict {
    fn get_as<T>(&'py self, key: &PyString) -> PyResult<Option<T>>
    where
        T: FromPyObject<'py>,
    {
        match self.get_item(key) {
            Some(t) => Ok(Some(<T>::extract(t)?)),
            None => Ok(None),
        }
    }

    fn get_as_req<T>(&'py self, key: &PyString) -> PyResult<T>
    where
        T: FromPyObject<'py>,
    {
        match self.get_item(key) {
            Some(t) => <T>::extract(t),
            None => py_err!(PyKeyError; "{}", key),
        }
    }
}

impl<'py> SchemaDict<'py> for Option<&PyDict> {
    fn get_as<T>(&'py self, key: &PyString) -> PyResult<Option<T>>
    where
        T: FromPyObject<'py>,
    {
        match self {
            Some(d) => d.get_as(key),
            None => Ok(None),
        }
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn get_as_req<T>(&'py self, key: &PyString) -> PyResult<T>
    where
        T: FromPyObject<'py>,
    {
        match self {
            Some(d) => d.get_as_req(key),
            None => py_err!(PyKeyError; "{}", key),
        }
    }
}

pub fn schema_or_config<'py, T>(
    schema: &'py PyDict,
    config: Option<&'py PyDict>,
    schema_key: &PyString,
    config_key: &PyString,
) -> PyResult<Option<T>>
where
    T: FromPyObject<'py>,
{
    match schema.get_as(schema_key)? {
        Some(v) => Ok(Some(v)),
        None => match config {
            Some(config) => config.get_as(config_key),
            None => Ok(None),
        },
    }
}

pub fn schema_or_config_same<'py, T>(
    schema: &'py PyDict,
    config: Option<&'py PyDict>,
    key: &PyString,
) -> PyResult<Option<T>>
where
    T: FromPyObject<'py>,
{
    schema_or_config(schema, config, key, key)
}

pub fn is_strict(schema: &PyDict, config: Option<&PyDict>) -> PyResult<bool> {
    let py = schema.py();
    Ok(schema_or_config_same(schema, config, intern!(py, "strict"))?.unwrap_or(false))
}

enum SchemaErrorInner {
    Message(String),
    Error(ValidationError),
}

// we could perhaps do clever things here to store each schema error, or have different types for the top
// level error group, and other errors, we could perhaps also support error groups!?
#[pyclass(extends=PyException, module="pydantic_core._pydantic_core")]
pub struct SchemaError {
    value: SchemaErrorInner,
}

impl fmt::Debug for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SchemaError({:?})", self.message())
    }
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message())
    }
}

impl Error for SchemaError {
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl SchemaError {
    pub fn new_err<A>(args: A) -> PyErr
    where
        A: PyErrArguments + Send + Sync + 'static,
    {
        PyErr::new::<SchemaError, A>(args)
    }

    pub fn from_val_error(py: Python, error: ValError) -> PyErr {
        let validation_error = ValidationError::from_val_error(py, "Schema".to_object(py), error, None);
        match validation_error.into_value(py).extract::<ValidationError>(py) {
            Ok(err) => {
                let schema_error = SchemaError {
                    value: SchemaErrorInner::Error(err),
                };
                match Py::new(py, schema_error) {
                    Ok(err) => PyErr::from_value(err.into_ref(py)),
                    Err(err) => err,
                }
            }
            Err(err) => err,
        }
    }

    fn message(&self) -> String {
        match &self.value {
            SchemaErrorInner::Message(message) => message.to_owned(),
            SchemaErrorInner::Error(_) => "<ValidationError>".to_owned(),
        }
    }
}

#[pymethods]
impl SchemaError {
    #[new]
    fn py_new(message: String) -> Self {
        Self {
            value: SchemaErrorInner::Message(message),
        }
    }

    fn __repr__(&self, py: Python) -> String {
        match &self.value {
            SchemaErrorInner::Message(message) => format!("SchemaError({message:?})"),
            SchemaErrorInner::Error(error) => error.display(py),
        }
    }

    fn __str__(&self, py: Python) -> String {
        match &self.value {
            SchemaErrorInner::Message(message) => message.to_owned(),
            SchemaErrorInner::Error(error) => error.display(py),
        }
    }

    fn error_count(&self) -> usize {
        match &self.value {
            SchemaErrorInner::Message(_) => 0,
            SchemaErrorInner::Error(error) => error.error_count(),
        }
    }

    fn errors(&self, py: Python) -> PyResult<Py<PyList>> {
        match &self.value {
            SchemaErrorInner::Message(_) => Ok(PyList::new(py, Vec::<PyAny>::new()).into_py(py)),
            SchemaErrorInner::Error(error) => error.errors(py, None),
        }
    }
}

macro_rules! py_error_type {
    ($msg:expr) => {
        crate::build_tools::py_error_type!(crate::build_tools::SchemaError; $msg)
    };
    ($msg:expr, $( $msg_args:expr ),+ ) => {
        crate::build_tools::py_error_type!(crate::build_tools::SchemaError; $msg, $( $msg_args ),+)
    };

    ($error_type:ty; $msg:expr) => {
        <$error_type>::new_err($msg)
    };

    ($error_type:ty; $msg:expr, $( $msg_args:expr ),+ ) => {
        <$error_type>::new_err(format!($msg, $( $msg_args ),+))
    };
}
pub(crate) use py_error_type;

macro_rules! py_err {
    ($msg:expr) => {
        Err(crate::build_tools::py_error_type!($msg))
    };
    ($msg:expr, $( $msg_args:expr ),+ ) => {
        Err(crate::build_tools::py_error_type!($msg, $( $msg_args ),+))
    };

    ($error_type:ty; $msg:expr) => {
        Err(crate::build_tools::py_error_type!($error_type; $msg))
    };

    ($error_type:ty; $msg:expr, $( $msg_args:expr ),+ ) => {
        Err(crate::build_tools::py_error_type!($error_type; $msg, $( $msg_args ),+))
    };
}
pub(crate) use py_err;

pub fn function_name(f: &PyAny) -> PyResult<String> {
    match f.getattr(intern!(f.py(), "__name__")) {
        Ok(name) => name.extract(),
        _ => f.repr()?.extract(),
    }
}

pub fn safe_repr(v: &PyAny) -> Cow<str> {
    if let Ok(s) = v.repr() {
        s.to_string_lossy()
    } else if let Ok(name) = v.get_type().name() {
        format!("<unprintable {name} object>").into()
    } else {
        "<unprintable object>".into()
    }
}
