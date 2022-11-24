use std::hash::{BuildHasher, BuildHasherDefault, Hash};

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PySet, PyTuple};

use nohash_hasher::{IntSet, NoHashHasher};
use pyo3::exceptions::PyTypeError;
use serde::ser::SerializeSeq;

use crate::build_tools::SchemaDict;

use super::any::{fallback_serialize, fallback_to_python, AnySerializer};
use super::shared::{py_err_se_err, BuildSerializer, CombinedSerializer, Extra, SerMode, TypeSerializer};
use super::PydanticSerializer;

#[derive(Debug, Clone, Default)]
pub struct SchemaIncEx<T> {
    include: Option<IntSet<T>>,
    exclude: Option<IntSet<T>>,
}

impl SchemaIncEx<usize> {
    pub fn new_from_ints(include: Option<&PyAny>, exclude: Option<&PyAny>) -> PyResult<Self> {
        let include = Self::build_set_ints(include)?;
        let exclude = Self::build_set_ints(exclude)?;
        Ok(Self { include, exclude })
    }

    fn build_set_ints(v: Option<&PyAny>) -> PyResult<Option<IntSet<usize>>> {
        match v {
            Some(value) => {
                if value.is_none() {
                    Ok(None)
                } else {
                    let py_set: &PySet = value.cast_as()?;
                    let mut set: IntSet<usize> =
                        IntSet::with_capacity_and_hasher(py_set.len(), BuildHasherDefault::default());

                    for item in py_set {
                        set.insert(item.extract()?);
                    }
                    Ok(Some(set))
                }
            }
            None => Ok(None),
        }
    }
}

impl SchemaIncEx<isize> {
    pub fn new_from_hash(include: Option<&PyAny>, exclude: Option<&PyAny>) -> PyResult<Self> {
        let include = Self::build_set_hashes(include)?;
        let exclude = Self::build_set_hashes(exclude)?;
        Ok(Self { include, exclude })
    }

    fn build_set_hashes(v: Option<&PyAny>) -> PyResult<Option<IntSet<isize>>> {
        match v {
            Some(value) => {
                if value.is_none() {
                    Ok(None)
                } else {
                    let py_set: &PySet = value.cast_as()?;
                    let mut set: IntSet<isize> =
                        IntSet::with_capacity_and_hasher(py_set.len(), BuildHasherDefault::default());

                    for item in py_set {
                        set.insert(item.hash()?);
                    }
                    Ok(Some(set))
                }
            }
            None => Ok(None),
        }
    }
}

impl<T> SchemaIncEx<T>
where
    T: Hash + Eq + Copy,
    BuildHasherDefault<NoHashHasher<T>>: BuildHasher,
{
    /// this is the somewhat hellish logic for deciding:
    /// 1. whether we should omit a value at a particular index/key - returning `Ok(None)` here
    /// 2. and if we are including it, what values of `include` and `exclude` should be passed to it
    pub fn include_or_exclude<'py>(
        &self,
        py_key: &PyAny,
        int_key: T,
        include: Option<&'py PyAny>,
        exclude: Option<&'py PyAny>,
    ) -> PyResult<Option<(Option<&'py PyAny>, Option<&'py PyAny>)>> {
        let mut next_exclude: Option<&PyAny> = None;
        if let Some(exclude) = exclude {
            if let Ok(exclude_dict) = exclude.cast_as::<PyDict>() {
                if let Some(exc_value) = exclude_dict.get_item(py_key) {
                    if exc_value.is_none() {
                        // if the index is in exclude, and the exclude value is `None`, we want to omit this index
                        return Ok(None);
                    } else {
                        // if the index is in exclude, and the exclude-value is not `None`,
                        // we want to return `Some((..., Some(next_exclude))`
                        next_exclude = Some(exc_value);
                    }
                }
            } else if let Ok(exclude_set) = exclude.cast_as::<PySet>() {
                // question: should we `unwrap_or(false)` instead of raise an error here?
                if exclude_set.contains(py_key)? {
                    // index is in the exclude set, we return Ok(None) to omit this index
                    return Ok(None);
                }
            } else if !exclude.is_none() {
                return Err(PyTypeError::new_err("`exclude` argument must a set or dict."));
            }
        }

        if let Some(include) = include {
            if let Ok(include_dict) = include.cast_as::<PyDict>() {
                if let Some(inc_value) = include_dict.get_item(py_key) {
                    // if the index is in include, we definitely want to include this index
                    return if inc_value.is_none() {
                        Ok(Some((None, next_exclude)))
                    } else {
                        Ok(Some((Some(inc_value), next_exclude)))
                    };
                } else if !self.in_include(int_key) {
                    // if the index is not in include, include exists, AND it's not in schema include,
                    // this index should be omitted
                    return Ok(None);
                }
            } else if let Ok(include_set) = include.cast_as::<PySet>() {
                // question: as above
                if include_set.contains(py_key)? {
                    return Ok(Some((None, next_exclude)));
                } else if !self.in_include(int_key) {
                    // if the index is not in include, include exists, AND it's not in schema include,
                    // this index should be omitted
                    return Ok(None);
                }
            } else if !include.is_none() {
                return Err(PyTypeError::new_err("`include` argument must a set or dict."));
            }
        }

        if next_exclude.is_some() {
            Ok(Some((None, next_exclude)))
        } else if self.default_include(int_key) {
            Ok(Some((None, None)))
        } else {
            Ok(None)
        }
    }

    /// default decision on whether to include the item at at given `index`/`key`
    pub fn default_include(&self, value: T) -> bool {
        match (&self.include, &self.exclude) {
            (Some(include), Some(exclude)) => include.contains(&value) && !exclude.contains(&value),
            (Some(include), None) => include.contains(&value),
            (None, Some(exclude)) => !exclude.contains(&value),
            (None, None) => true,
        }
    }

    /// whether an `index`/`key` is explicitly included, this is combined with call-time `include` below
    pub fn in_include(&self, value: T) -> bool {
        match self.include {
            Some(ref include) => include.contains(&value),
            None => false,
        }
    }
}

macro_rules! build_serializer {
    ($struct_name:ident, $expected_type:literal) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name {
            item_serializer: Box<CombinedSerializer>,
            inc_ex: SchemaIncEx<usize>,
        }

        impl BuildSerializer for $struct_name {
            const EXPECTED_TYPE: &'static str = $expected_type;

            fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
                let py = schema.py();
                let item_serializer = match schema.get_as::<&PyDict>(intern!(py, "items_schema"))? {
                    Some(items_schema) => CombinedSerializer::build(items_schema, config)?,
                    None => AnySerializer::build(schema, config)?,
                };
                let inc_ex = match schema.get_as::<&PyDict>(intern!(py, "serialization"))? {
                    Some(ser) => {
                        let include = ser.get_item(intern!(py, "include"));
                        let exclude = ser.get_item(intern!(py, "exclude"));
                        SchemaIncEx::new_from_ints(include, exclude)?
                    }
                    None => SchemaIncEx::default(),
                };
                Ok(Self {
                    item_serializer: Box::new(item_serializer),
                    inc_ex,
                }
                .into())
            }
        }

        impl $struct_name {
            fn include_or_exclude<'s, 'py>(
                &'s self,
                py: Python<'py>,
                index: usize,
                include: Option<&'py PyAny>,
                exclude: Option<&'py PyAny>,
            ) -> PyResult<Option<(Option<&'py PyAny>, Option<&'py PyAny>)>> {
                let py_key = index.to_object(py);
                self.inc_ex
                    .include_or_exclude(py_key.as_ref(py), index, include, exclude)
            }
        }
    };
}

build_serializer!(ListSerializer, "list");

impl TypeSerializer for ListSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        match value.cast_as::<PyList>() {
            Ok(py_list) => {
                let py = value.py();
                let item_serializer = self.item_serializer.as_ref();

                let mut items = Vec::with_capacity(py_list.len());
                for (index, element) in py_list.iter().enumerate() {
                    if let Some((next_include, next_exclude)) = self.include_or_exclude(py, index, include, exclude)? {
                        items.push(item_serializer.to_python(element, next_include, next_exclude, extra)?);
                    }
                }
                Ok(items.into_py(py))
            }
            Err(_) => {
                extra.warnings.fallback_filtering(Self::EXPECTED_TYPE, value);
                fallback_to_python(value, extra)
            }
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        match value.cast_as::<PyList>() {
            Ok(py_list) => {
                let mut seq = serializer.serialize_seq(Some(py_list.len()))?;
                let item_serializer = self.item_serializer.as_ref();

                for (index, value) in py_list.iter().enumerate() {
                    if let Some((next_include, next_exclude)) = self
                        .include_or_exclude(value.py(), index, include, exclude)
                        .map_err(py_err_se_err)?
                    {
                        let item_serialize =
                            PydanticSerializer::new(value, item_serializer, next_include, next_exclude, extra);
                        seq.serialize_element(&item_serialize)?;
                    }
                }
                seq.end()
            }
            Err(_) => {
                extra.warnings.fallback_filtering(Self::EXPECTED_TYPE, value);
                fallback_serialize(value, serializer, extra.ob_type_lookup)
            }
        }
    }
}

build_serializer!(TupleSerializer, "tuple");

impl TypeSerializer for TupleSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        match value.cast_as::<PyTuple>() {
            Ok(py_tuple) => {
                let py = value.py();
                let item_serializer = self.item_serializer.as_ref();

                let mut items = Vec::with_capacity(py_tuple.len());
                for (index, element) in py_tuple.iter().enumerate() {
                    if let Some((next_include, next_exclude)) = self.include_or_exclude(py, index, include, exclude)? {
                        items.push(item_serializer.to_python(element, next_include, next_exclude, extra)?);
                    }
                }
                match extra.mode {
                    SerMode::Json => Ok(PyList::new(py, items).into_py(py)),
                    _ => Ok(PyTuple::new(py, items).into_py(py)),
                }
            }
            Err(_) => {
                extra.warnings.fallback_filtering(Self::EXPECTED_TYPE, value);
                fallback_to_python(value, extra)
            }
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        match value.cast_as::<PyTuple>() {
            Ok(py_tuple) => {
                let py_tuple: &PyTuple = py_tuple.cast_as().map_err(py_err_se_err)?;
                let item_serializer = self.item_serializer.as_ref();

                let mut seq = serializer.serialize_seq(Some(py_tuple.len()))?;
                for (index, value) in py_tuple.iter().enumerate() {
                    if let Some((next_include, next_exclude)) = self
                        .include_or_exclude(value.py(), index, include, exclude)
                        .map_err(py_err_se_err)?
                    {
                        let item_serialize =
                            PydanticSerializer::new(value, item_serializer, next_include, next_exclude, extra);
                        seq.serialize_element(&item_serialize)?;
                    }
                }
                seq.end()
            }
            Err(_) => {
                extra.warnings.fallback_filtering(Self::EXPECTED_TYPE, value);
                fallback_serialize(value, serializer, extra.ob_type_lookup)
            }
        }
    }
}
