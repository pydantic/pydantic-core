use std::borrow::Cow;
use std::fmt;

use pyo3::prelude::*;

/// Used to store individual items of the error location and also in lookup keys, e.g. a string for key/field names
/// or a number for array indices.
#[derive(Debug, Clone)]
pub enum LocItem<'a> {
    /// string type key, used to get or identify items from a dict or anything that implements `__getitem__`
    S(Cow<'a, str>),
    /// integer key, used to get items from a list, tuple OR a dict with int keys `Dict[int, ...]` (python only)
    I(usize),
}

impl fmt::Display for LocItem<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocItem::S(s) => write!(f, "{}", s),
            LocItem::I(i) => write!(f, "{}", i),
        }
    }
}

impl ToPyObject for LocItem<'_> {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            Self::S(val) => val.to_object(py),
            Self::I(val) => val.to_object(py),
        }
    }
}

impl From<String> for LocItem<'_> {
    fn from(s: String) -> Self {
        Self::S(Cow::Owned(s))
    }
}

impl<'a> From<&'a str> for LocItem<'a> {
    fn from(s: &'a str) -> Self {
        Self::S(Cow::Borrowed(s))
    }
}

impl From<usize> for LocItem<'_> {
    fn from(i: usize) -> Self {
        Self::I(i)
    }
}

/// Error locations are represented by a vector of `LocItem`s.
/// e.g. if the error occurred in the third member of a list called `foo`,
/// the location would be `["foo", 2]`.
pub type Location<'a> = Vec<LocItem<'a>>;

pub fn owned_location<'a, 'b>(loc: &'a Location) -> Location<'b> {
    loc.iter()
        .map(|item| match item {
            LocItem::S(s) => LocItem::S(Cow::Owned(s.to_string())),
            LocItem::I(i) => LocItem::I(*i),
        })
        .collect()
}
