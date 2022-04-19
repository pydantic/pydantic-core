use std::fmt;

use pyo3::prelude::*;

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

impl ToPy for &str {
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

    fn validate_dict<'py>(&'py self, py: Python<'py>) -> ValResult<Box<dyn DictInput<'py> + 'py>>;

    fn validate_list<'py>(&'py self, py: Python<'py>) -> ValResult<Box<dyn ListInput<'py> + 'py>>;
}

pub trait DictInput<'py>: ToPy {
    fn iter(&self) -> Box<dyn Iterator<Item = (&dyn Input, &dyn Input)> + '_>;
    fn get_item(&self, key: &str) -> Option<&'_ dyn Input>;
    fn len(&self) -> usize;
}

pub trait ListInput<'py>: ToPy {
    fn iter(&self) -> Box<dyn Iterator<Item = &dyn Input> + '_>;
    fn len(&self) -> usize;
}
