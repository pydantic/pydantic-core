use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use crate::input::Input;

use super::{ErrorKind, ValError};

#[pyclass(extends=PyValueError, module="pydantic_core._pydantic_core")]
#[derive(Debug, Clone)]
pub struct PydanticValueError {
    kind: String,
    message_template: String,
    context: Option<Py<PyDict>>,
}

#[pymethods]
impl PydanticValueError {
    #[new]
    fn py_new(py: Python, kind: String, message_template: String, context: Option<&PyDict>) -> Self {
        Self {
            kind,
            message_template,
            context: context.map(|c| c.into_py(py)),
        }
    }

    #[getter]
    pub fn kind(&self) -> String {
        self.kind.clone()
    }

    pub fn message(&self, py: Python) -> String {
        let mut message = self.message_template.clone();
        if let Some(ref context) = self.context {
            for item in context.as_ref(py).items().iter() {
                let (key, value): (&PyString, &PyString) = item.extract().unwrap();
                message = message.replace(
                    &format!("{{{}}}", key.to_string_lossy().as_ref()),
                    value.to_string_lossy().as_ref(),
                );
            }
        }
        return message;
    }

    fn __repr__(&self, py: Python) -> String {
        format!("{} [kind={}]", self.message(py), self.kind)
    }

    fn __str__(&self, py: Python) -> String {
        self.__repr__(py)
    }
}

impl PydanticValueError {
    pub fn into_val_error<'a>(self, input: &'a impl Input<'a>) -> ValError<'a> {
        let kind = ErrorKind::CustomError { value_error: self };
        ValError::new(kind, input)
    }

    pub fn get_kind(&self) -> String {
        self.kind.clone()
    }

    pub fn get_context(&self, py: Python) -> Option<Py<PyDict>> {
        self.context.as_ref().map(|c| c.clone_ref(py))
    }
}
