use std::fmt;
use std::fmt::Debug;

use enum_dispatch::enum_dispatch;
use pyo3::prelude::*;
use pyo3::types::PyType;

use super::generic_dict::GenericDict;
use super::generic_list::ListInput;
use super::parse_json::JsonInput;
use crate::errors::{LocItem, ValResult};

#[enum_dispatch]
#[derive(Debug)]
pub enum CombinedInput<'data> {
    Py(&'data PyAny),
    Json(&'data JsonInput),
}

#[enum_dispatch(CombinedInput)]
pub trait ToPy: Debug {
    fn to_py(&self, py: Python) -> PyObject;
}

#[enum_dispatch(CombinedInput)]
pub trait ToLocItem {
    fn to_loc(&self) -> LocItem;
}

impl ToLocItem for String {
    fn to_loc(&self) -> LocItem {
        LocItem::S(self.clone())
    }
}

impl ToLocItem for &String {
    fn to_loc(&self) -> LocItem {
        LocItem::S(self.to_string())
    }
}

impl ToLocItem for &str {
    fn to_loc(&self) -> LocItem {
        LocItem::S(self.to_string())
    }
}

#[enum_dispatch(CombinedInput)]
pub trait Input<'data>: fmt::Debug + ToPy + ToLocItem {
    fn is_none(&self) -> bool;

    fn strict_str(&self) -> ValResult<String>;

    fn lax_str(&self) -> ValResult<String>;

    fn strict_bool(&self) -> ValResult<bool>;

    fn lax_bool(&self) -> ValResult<bool>;

    fn strict_int(&self) -> ValResult<i64>;

    fn lax_int(&self) -> ValResult<i64>;

    fn strict_float(&self) -> ValResult<f64>;

    fn lax_float(&self) -> ValResult<f64>;

    fn strict_model_check(&self, class: &PyType) -> ValResult<bool>;

    fn strict_dict(&'data self) -> ValResult<GenericDict<'data>>;

    fn lax_dict(&'data self, _try_instance: bool) -> ValResult<GenericDict<'data>> {
        self.strict_dict()
    }

    fn strict_list(&'data self) -> ValResult<ListInput<'data>>;

    fn lax_list(&'data self) -> ValResult<ListInput<'data>> {
        self.strict_list()
    }

    fn strict_set(&'data self) -> ValResult<ListInput<'data>>;

    fn lax_set(&'data self) -> ValResult<ListInput<'data>> {
        self.strict_set()
    }
}
