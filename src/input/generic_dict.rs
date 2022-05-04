use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyFrozenSet, PyInt, PyList, PyMapping, PySet, PyString, PyTuple, PyType};

use super::parse_json::JsonObject;
use super::traits::{CombinedInput, Input, ToLocItem, ToPy};

pub enum GenericDict<'data> {
    Py(&'data PyDict),
    Json(&'data JsonObject),
}

impl<'d> From<&'d PyDict> for GenericDict<'d> {
    fn from(d: &'d PyDict) -> Self {
        Self::Py(d)
    }
}

impl<'d> From<&'d JsonObject> for GenericDict<'d> {
    fn from(d: &'d JsonObject) -> Self {
        Self::Json(d)
    }
}

impl<'data> GenericDict<'data> {
    pub fn input_iter(&self) -> Box<dyn Iterator<Item = (CombinedInput<'data>, CombinedInput<'data>)> + 'data> {
        match self {
            Self::Py(list) => Box::new(list.iter().map(|(k, v)| (k.into(), v.into()))),
            Self::Json(json) => Box::new(json.iter().map(|(k, v)| (k.into(), v.into()))),
        }
    }

    pub fn input_get(&self, key: &str) -> Option<CombinedInput<'data>> {
        match self {
            Self::Py(list) => list.get_item(key).map(|v| v.into()),
            Self::Json(json) => json.get(key).map(|v| v.into()),
        }
    }

    pub fn input_len(&self) -> usize {
        match self {
            Self::Py(d) => d.len(),
            Self::Json(d) => d.len(),
        }
    }
}
