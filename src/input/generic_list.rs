use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyFrozenSet, PyInt, PyList, PyMapping, PySet, PyString, PyTuple, PyType};

use super::parse_json::JsonArray;
use super::traits::{CombinedInput, Input, ToLocItem, ToPy};

pub enum ListInput<'data> {
    PyList(&'data PyList),
    PySet(&'data PySet),
    PyTuple(&'data PyTuple),
    PyFrozenSet(&'data PyFrozenSet),
    Json(&'data JsonArray),
}

impl<'d> From<&'d PyList> for ListInput<'d> {
    fn from(d: &'d PyList) -> Self {
        Self::PyList(d)
    }
}

impl<'d> From<&'d PySet> for ListInput<'d> {
    fn from(d: &'d PySet) -> Self {
        Self::PySet(d)
    }
}

impl<'d> From<&'d PyTuple> for ListInput<'d> {
    fn from(d: &'d PyTuple) -> Self {
        Self::PyTuple(d)
    }
}

impl<'d> From<&'d PyFrozenSet> for ListInput<'d> {
    fn from(d: &'d PyFrozenSet) -> Self {
        Self::PyFrozenSet(d)
    }
}

impl<'d> From<&'d JsonArray> for ListInput<'d> {
    fn from(d: &'d JsonArray) -> Self {
        Self::Json(d)
    }
}

impl<'data> ListInput<'data> {
    pub fn input_iter(&self) -> Box<dyn Iterator<Item = CombinedInput<'data>> + 'data> {
        match self {
            Self::PyList(list) => Box::new(list.iter().map(|item| item.into())),
            Self::PySet(set) => Box::new(set.iter().map(|item| item.into())),
            Self::PyTuple(tuple) => Box::new(tuple.iter().map(|item| item.into())),
            Self::PyFrozenSet(s) => Box::new(s.iter().map(|item| item.into())),
            Self::Json(json) => Box::new(json.iter().map(|item| item.into())),
        }
    }

    pub fn input_len(&self) -> usize {
        match self {
            Self::PyList(list) => list.len(),
            Self::PySet(set) => set.len(),
            Self::PyTuple(tuple) => tuple.len(),
            Self::PyFrozenSet(set) => set.len(),
            Self::Json(json) => json.len(),
        }
    }
}
