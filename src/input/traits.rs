use std::fmt;

use pyo3::prelude::*;
use pyo3::types::{PyBool, PyFloat, PyString};

use crate::errors::{LocItem, ValResult};

pub trait ToPy {
    fn to_pyany<'py>(&'py self, py: Python<'py>) -> &'py PyAny;
}

/// special cases of standard types that need to implement ToPy
impl ToPy for String {
    fn to_pyany<'py>(&'py self, py: Python<'py>) -> &'py PyAny {
        PyString::new(py, self.as_str()).as_ref()
    }
}

impl ToPy for &str {
    fn to_pyany<'py>(&'py self, py: Python<'py>) -> &'py PyAny {
        PyString::new(py, self).as_ref()
    }
}

impl ToPy for i64 {
    fn to_pyany<'py>(&'py self, py: Python<'py>) -> &'py PyAny {
        let py_int = self.into_py(py);
        py_int.into_ref(py)
    }
}

impl ToPy for f64 {
    fn to_pyany<'py>(&'py self, py: Python<'py>) -> &'py PyAny {
        PyFloat::new(py, *self).as_ref()
    }
}

impl ToPy for bool {
    fn to_pyany<'py>(&'py self, py: Python<'py>) -> &'py PyAny {
        PyBool::new(py, *self).as_ref()
    }
}

pub trait ToLocItem {
    fn to_loc(&self) -> ValResult<LocItem>;
}

impl ToLocItem for String {
    fn to_loc(&self) -> ValResult<LocItem> {
        Ok(LocItem::S(self.clone()))
    }
}

impl ToLocItem for &str {
    fn to_loc(&self) -> ValResult<LocItem> {
        Ok(LocItem::S(self.to_string()))
    }
}

pub trait Input: fmt::Debug + ToPy + ToLocItem {
    fn validate_none(&self, py: Python) -> ValResult<()>;

    fn validate_str(&self, py: Python) -> ValResult<String>;

    fn validate_bool(&self, py: Python) -> ValResult<bool>;

    fn validate_int(&self, py: Python) -> ValResult<i64>;

    fn validate_float(&self, py: Python) -> ValResult<f64>;

    fn validate_dict<'py>(&'py self, py: Python<'py>) -> ValResult<Box<dyn DictInput<'py> + 'py>>;

    fn validate_list<'py>(&'py self, py: Python<'py>) -> ValResult<Box<dyn ListInput<'py> + 'py>>;
}

// these are ugly, is there any way to avoid the maps in iter, one of the boxes and/or the duplication?
// is this harming performance, particularly the .map(|item| item)?
// https://stackoverflow.com/a/47156134/949890
pub trait DictInput<'py>: ToPy {
    fn input_iter(&self) -> Box<dyn Iterator<Item = (&dyn Input, &dyn Input)> + '_>;

    fn input_get(&self, key: &str) -> Option<&'_ dyn Input>;

    fn input_len(&self) -> usize;
}

pub trait ListInput<'py>: ToPy {
    fn input_iter(&self) -> Box<dyn Iterator<Item = &dyn Input> + '_>;

    fn input_len(&self) -> usize;
}
