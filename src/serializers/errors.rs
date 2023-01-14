use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::fmt;

static UNEXPECTED_TYPE_SER: &str = "__PydanticSerializationUnexpectedValue__";

pub(super) fn py_err_se_err<T: serde::ser::Error, E: fmt::Display>(py_error: E) -> T {
    T::custom(py_error.to_string())
}

pub(super) fn se_err_py_err(error: serde_json::Error) -> PyErr {
    let s = error.to_string();
    return if s == UNEXPECTED_TYPE_SER {
        PydanticSerializationUnexpectedValue::new_err()
    } else {
        let msg = format!("Error serializing to JSON: {s}");
        PydanticSerializationError::new_err(msg)
    };
}

#[pyclass(extends=PyValueError, module="pydantic_core._pydantic_core")]
#[derive(Debug, Clone)]
pub struct PydanticSerializationError {
    message: String,
}

impl PydanticSerializationError {
    pub(crate) fn new_err(msg: String) -> PyErr {
        PyErr::new::<Self, String>(msg)
    }
}

#[pymethods]
impl PydanticSerializationError {
    #[new]
    fn py_new(message: String) -> Self {
        Self { message }
    }

    fn __str__(&self) -> &str {
        &self.message
    }

    fn __repr__(&self) -> String {
        format!("PydanticSerializationError({})", self.message)
    }
}

#[pyclass(extends=PyValueError, module="pydantic_core._pydantic_core")]
#[derive(Debug, Clone)]
pub struct PydanticSerializationUnexpectedValue {
    message: Option<String>,
}

impl PydanticSerializationUnexpectedValue {
    pub(crate) fn new_err() -> PyErr {
        PyErr::new::<Self, Option<String>>(None)
    }
}

#[pymethods]
impl PydanticSerializationUnexpectedValue {
    #[new]
    fn py_new(message: Option<String>) -> Self {
        Self { message }
    }

    fn __str__(&self) -> &str {
        match self.message {
            Some(ref s) => s,
            None => "Unexpected Value",
        }
    }

    fn __repr__(&self) -> String {
        format!("PydanticSerializationUnexpectedValue({})", self.__str__())
    }
}
