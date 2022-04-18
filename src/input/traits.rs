use std::fmt;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::errors::ValResult;

pub trait ToPy {
    fn to_py(&self, py: Python) -> PyObject;
}

/// special cases of standard types that need to implement ToPy
impl ToPy for String {
    fn to_py(&self, py: Python) -> PyObject {
        self.into_py(py)
    }
}

impl ToPy for i64 {
    fn to_py(&self, py: Python) -> PyObject {
        self.into_py(py)
    }
}

impl ToPy for f64 {
    fn to_py(&self, py: Python) -> PyObject {
        self.into_py(py)
    }
}

pub trait Input: fmt::Debug + ToPy {
    fn validate_none(&self, py: Python) -> ValResult<()>;

    fn validate_str(&self, py: Python) -> ValResult<String>;

    fn validate_bool(&self, py: Python) -> ValResult<bool>;

    fn validate_int(&self, py: Python) -> ValResult<i64>;

    fn validate_float(&self, py: Python) -> ValResult<f64>;

    fn validate_dict<'py>(&'py self, py: Python<'py>) -> ValResult<&'py PyDict>;
    // fn validate_dict<'py>(&'py self, py: Python<'py>) -> ValResult<&'py dyn InputDict<dyn Input, dyn Input>>;

    fn validate_list<'py>(&'py self, py: Python<'py>) -> ValResult<&'py PyList>;
    // fn validate_list<'py>(&'py self, py: Python<'py>) -> ValResult<&'py dyn InputList>;
}
