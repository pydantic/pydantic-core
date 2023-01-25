use std::hash::Hash;

use pyo3::exceptions::{PyKeyError, PyTypeError};
use pyo3::ffi::Py_Ellipsis;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PySet, PyString};
use pyo3::{intern, AsPyPointer};

use ahash::{AHashSet, AHashMap};

use crate::build_tools::SchemaDict;

#[derive(Debug, Clone, Default)]
pub(super) struct SchemaFilter<T> {
    include: Option<AHashSet<T>>,
    exclude: Option<AHashSet<T>>,
}

impl SchemaFilter<usize> {
    pub fn from_schema(schema: &PyDict) -> PyResult<Self> {
        let py = schema.py();
        match schema.get_as::<&PyDict>(intern!(py, "serialization"))? {
            Some(ser) => {
                let include = Self::build_set_ints(ser.get_item(intern!(py, "include")))?;
                let exclude = Self::build_set_ints(ser.get_item(intern!(py, "exclude")))?;
                Ok(Self { include, exclude })
            }
            None => Ok(SchemaFilter::default()),
        }
    }

    fn build_set_ints(v: Option<&PyAny>) -> PyResult<Option<AHashSet<usize>>> {
        match v {
            Some(value) => {
                if value.is_none() {
                    Ok(None)
                } else {
                    let py_set: &PySet = value.downcast()?;
                    let mut set: AHashSet<usize> = AHashSet::with_capacity(py_set.len());

                    for item in py_set {
                        set.insert(item.extract()?);
                    }
                    Ok(Some(set))
                }
            }
            None => Ok(None),
        }
    }

    pub fn value_filter<'py>(
        &self,
        index: usize,
        include: &'py FilterValue<'py>,
        exclude: &'py FilterValue<'py>,
    ) -> PyResult<Option<(&'py FilterValue<'py>, &'py FilterValue<'py>)>> {
        self.filter(index, index, include, exclude)
    }
}

impl SchemaFilter<isize> {
    pub fn from_set_hash(include: Option<&PyAny>, exclude: Option<&PyAny>) -> PyResult<Self> {
        let include = Self::build_set_hashes(include)?;
        let exclude = Self::build_set_hashes(exclude)?;
        Ok(Self { include, exclude })
    }

    pub fn from_vec_hash(py: Python, exclude: Vec<Py<PyString>>) -> PyResult<Self> {
        let exclude = if exclude.is_empty() {
            None
        } else {
            let mut set: AHashSet<isize> = AHashSet::with_capacity(exclude.len());
            for item in exclude {
                set.insert(item.as_ref(py).hash()?);
            }
            Some(set)
        };
        Ok(Self { include: None, exclude })
    }

    fn build_set_hashes(v: Option<&PyAny>) -> PyResult<Option<AHashSet<isize>>> {
        match v {
            Some(value) => {
                if value.is_none() {
                    Ok(None)
                } else {
                    let py_set: &PySet = value.downcast()?;
                    let mut set: AHashSet<isize> = AHashSet::with_capacity(py_set.len());

                    for item in py_set {
                        set.insert(item.hash()?);
                    }
                    Ok(Some(set))
                }
            }
            None => Ok(None),
        }
    }

    pub fn key_filter<'py>(
        &self,
        key: &PyAny,
        include: &'py FilterValue<'py>,
        exclude: &'py FilterValue<'py>,
    ) -> PyResult<Option<(&'py FilterValue<'py>, &'py FilterValue<'py>)>> {
        let hash = key.hash()?;
        self.filter(key, hash, include, exclude)
    }
}

fn is_ellipsis(v: &PyAny) -> bool {
    unsafe { v.as_ptr() == Py_Ellipsis() }
}

trait FilterLogic<T: Eq + Copy> {
    /// whether an `index`/`key` is explicitly included, this is combined with call-time `include` below
    fn explicit_include(&self, value: T) -> bool;
    /// default decision on whether to include the item at a given `index`/`key`
    fn default_filter(&self, value: T) -> bool;

    /// this is the somewhat hellish logic for deciding:
    /// 1. whether we should omit a value at a particular index/key - returning `Ok(None)` here
    /// 2. or include it, in which case, what values of `include` and `exclude` should be passed to it
    fn filter<'py>(
        &self,
        into_key: impl ToPyObject + Copy,
        int_key: T,
        include: &'py FilterValue<'py>,
        exclude: &'py FilterValue<'py>,
    ) -> PyResult<Option<(&'py FilterValue<'py>, &'py FilterValue<'py>)>> {
        let key = into_key.into();
        let next_exclude: &FilterValue = exclude.lookup(&key);
        if next_exclude == FilterValue::Ellipsis {
            // if the index is in exclude, and the exclude value is `...`, we want to omit this index/item
            return Ok(None);
        }

        let next_include: &FilterValue = include.lookup(&key);
        if include != FilterValue::None && next_include == FilterValue::None && !self.explicit_include(int_key) {
            // if the index is not in include, include exists, AND it's not in schema include,
            // this index should be omitted
            return Ok(None);
        } else if matches!(next_include, FilterValue::Ellipsis | FilterValue::AllMap(_)) {
            // if the index is in include, we definitely want to include this index/item
            Ok(Some((next_include, next_exclude)))
        } else if matches!(next_include, FilterValue::AllMap(_)) {
            // exclude exists and is not `...`, so we want to include this index/item
            Ok(Some((next_include, next_exclude)))
        } else if self.default_filter(int_key) {
            // otherwise we fallback to the `default_filter`
            Ok(Some((&FilterValue::None, &FilterValue::None)))
        } else {
            Ok(None)
        }
    }
}

impl<T> FilterLogic<T> for SchemaFilter<T>
where
    T: Hash + Eq + Copy,
{
    fn explicit_include(&self, value: T) -> bool {
        match self.include {
            Some(ref include) => include.contains(&value),
            None => false,
        }
    }

    fn default_filter(&self, value: T) -> bool {
        match (&self.include, &self.exclude) {
            (Some(include), Some(exclude)) => include.contains(&value) && !exclude.contains(&value),
            (Some(include), None) => include.contains(&value),
            (None, Some(exclude)) => !exclude.contains(&value),
            (None, None) => true,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct AnyFilter;

impl AnyFilter {
    pub fn new() -> Self {
        AnyFilter {}
    }

    pub fn key_filter<'py>(
        &self,
        key: &PyAny,
        include: &'py FilterValue<'py>,
        exclude: &'py FilterValue<'py>,
    ) -> PyResult<Option<(&'py FilterValue<'py>, &'py FilterValue<'py>)>> {
        // just use 0 for the int_key, it's always ignored in the implementation here
        self.filter(key, 0, include, exclude)
    }

    pub fn value_filter<'py>(
        &self,
        index: usize,
        include: &'py FilterValue<'py>,
        exclude: &'py FilterValue<'py>,
    ) -> PyResult<Option<(&'py FilterValue<'py>, &'py FilterValue<'py>)>> {
        self.filter(index, index, include, exclude)
    }
}

impl<T> FilterLogic<T> for AnyFilter
where
    T: Eq + Copy,
{
    fn explicit_include(&self, _value: T) -> bool {
        false
    }

    fn default_filter(&self, _value: T) -> bool {
        true
    }
}

// fn merge_all_value(item_value: Option<&PyAny>, all_value: Option<&PyAny>) -> Option<&PyDict> {
//     match (item_value, all_value) {
// (Some(item_value), Some(all_value)) => {
//             let py = item_value.py();
//             let merged = PyDict::new(py);
//             merged.merge(all_value).unwrap();
//             merged.merge(item_value).unwrap();
//             Some(merged)
//         }
//         (Some(item_value), None) => item_value.downcast().ok(),
//         (None, Some(all_value)) => all_value.downcast().ok(),
//         (None, None) => None,
//     }
// }
//
// fn as_dict(value: &PyAny) -> &PyDict {
//
// }

#[derive(Debug, Eq, PartialEq)]
pub(super) enum FilterKey<'py> {
    Int(i64),
    Str(&'py str)
}

impl<'py> FilterKey<'py> {
    fn from_py(py_key: impl ToPyObject + Copy) -> PyResult<Self> {
        let py_key = py_key.to_object(py_key.py());
        if let Ok(i) = py_key.extract::<i64>(py_key.py()) {
            Ok(Self::Int(i))
        } else if let Ok(py_str) = py_key.downcast::<PyString>(py_key.py()) {
            let str_key = py_str.to_str()?;
            Ok(Self::Str(str_key))
        } else {
            Err(PyKeyError::new_err("Filter keys must be integers or strings"))
        }
    }
}

impl<'py> From<&'py str> for FilterKey<'py> {
    fn from(s: &'py str) -> Self {
        Self::Str(s)
    }
}

impl<'py> From<i64> for FilterKey<'py> {
    fn from(i: i64) -> Self {
        Self::Int(i)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) enum FilterValue<'py> {
    None,
    Ellipsis,
    AllMap((Box<FilterValue<'py>>, AHashMap<FilterKey<'py>, FilterValue<'py>>)),
}

impl<'py> FilterValue<'py> {
    pub(super) fn new(arg: Option<&'py PyAny>) -> PyResult<Self> {
        match arg {
            Some(arg) => Self::build_recursive(arg),
            None => Ok(Self::None),
        }
    }

    fn lookup(&self, key: &FilterKey<'py>) -> &Self {
        match self {
            Self::AllMap((all, map)) => match map.get(key) {
                Some(value) => value,
                None => all.lookup(key),
            }
            _ => self,
        }
    }

    fn build_recursive(arg: &'py PyAny) -> PyResult<Self> {
        if is_ellipsis(arg) || arg.extract::<bool>() == Ok(true) {
            Ok(Self::Ellipsis)
        } else if let Ok(arg_set) = arg.downcast::<PySet>() {
            let mut map = AHashMap::with_capacity(arg_set.len());
            let mut all = FilterValue::None;
            for entry in arg_set {
                if let Ok(key) = entry.extract::<i64>() {
                    map.insert(key.into(), Self::Ellipsis);
                } else if let Ok(py_key) = entry.downcast::<PyString>() {
                    let str_key = py_key.to_str()?;
                    if str_key == "__all__" {
                        all = Self::Ellipsis;
                    } else {
                        map.insert(str_key.into(), Self::Ellipsis);
                    }
                } else {
                   return Err(PyTypeError::new_err("`include` and `exclude` set values must be ints or strings."));
                }
            }
            Ok(Self::AllMap((Box::new(all), map)))
        } else if let Ok(arg_dict) = arg.downcast::<PyDict>() {
            let mut map = AHashMap::with_capacity(arg_dict.len());
            let mut all = FilterValue::None;
            for (key, value) in arg_dict {
                let filter_value = Self::build_recursive(value)?;
                if let Ok(key) = key.extract::<i64>() {
                    map.insert(FilterKey::Int(key), filter_value);
                } else if let Ok(py_key) = key.downcast::<PyString>() {
                    let str_key = py_key.to_str()?;
                    if str_key == "__all__" {
                        all = filter_value;
                    } else {
                        map.insert(FilterKey::Str(str_key), filter_value);
                    }
                } else {
                   return Err(PyTypeError::new_err("`include` and `exclude` keys must be ints or strings."));
                }
            }
            Ok(Self::AllMap((Box::new(all), map)))
        } else {
           Err(PyTypeError::new_err("`include` and `exclude` arguments must a set, dict or None."))
        }
    }
}
