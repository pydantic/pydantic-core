use std::borrow::Cow;
use std::hash::BuildHasherDefault;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PySet, PyTuple};

use nohash_hasher::IntMap;
use serde::ser::SerializeSeq;

use super::any::{AnySerializer, ObTypeLookup};
use super::{build_serializer, py_err_se_err, BuildSerializer, CombinedSerializer, PydanticSerializer, TypeSerializer};
use crate::build_tools::SchemaDict;

type IncEx = Option<IntMap<usize, Option<PyObject>>>;

pub fn to_inc_ex(value: Option<&PyAny>) -> PyResult<IncEx> {
    match value {
        Some(value) => {
            if let Ok(py_set) = value.cast_as::<PySet>() {
                let mut map: IntMap<usize, Option<PyObject>> =
                    IntMap::with_capacity_and_hasher(py_set.len(), BuildHasherDefault::default());
                for item in py_set {
                    map.insert(item.extract()?, None);
                }
                Ok(Some(map))
            } else {
                let py = value.py();
                let py_dict: &PyDict = value.cast_as()?;
                let mut map: IntMap<usize, Option<PyObject>> =
                    IntMap::with_capacity_and_hasher(py_dict.len(), BuildHasherDefault::default());
                for (key, value) in py_dict {
                    // I'm using `None` here to replace what `'__all__'` meant in v1
                    let value = if value.is_none() {
                        None
                    } else {
                        Some(value.to_object(py))
                    };
                    map.insert(key.extract()?, value);
                }
                Ok(Some(map))
            }
        }
        None => Ok(None),
    }
}

macro_rules! build_sequence_serializer {
    ($struct_name:ident, $expected_type:literal, $type_:ty) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name {
            item_serializer: Box<CombinedSerializer>,
            include: IncEx,
            exclude: IncEx,
        }

        impl BuildSerializer for $struct_name {
            const EXPECTED_TYPE: &'static str = $expected_type;

            fn build_combined(schema: &PyDict, config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
                let py = schema.py();
                let item_serializer = match schema.get_as::<&PyDict>(pyo3::intern!(py, "items_schema"))? {
                    Some(items_schema) => build_serializer(items_schema, config)?,
                    None => AnySerializer::build_combined(schema, config)?,
                };
                Ok(Self {
                    item_serializer: Box::new(item_serializer),
                    include: to_inc_ex(schema.get_item(pyo3::intern!(py, "include")))?,
                    exclude: to_inc_ex(schema.get_item(pyo3::intern!(py, "exclude")))?,
                }
                .into())
            }
        }
    };
}

/// combine the validation time include/exclude with the include/exclude when creating the serializer
/// **NOTE:** we merge with union for both include and exclude, this is a change from V1 where we did,
/// union for exclude and intersection for include
fn union_inc_ex<'py>(
    py: Python<'py>,
    val_time_arg: Option<&'py PyAny>,
    self_inc_ex: &'py IncEx,
) -> PyResult<Cow<'py, IncEx>> {
    match to_inc_ex(val_time_arg)? {
        Some(mut inc_ex) => {
            if let Some(self_inc_ex) = self_inc_ex {
                // this is a union, not an intersection!
                for (key, value) in self_inc_ex {
                    inc_ex
                        .entry(*key)
                        .or_insert_with(|| value.as_ref().map(|value| value.clone_ref(py)));
                }
            }
            Ok(Cow::Owned(Some(inc_ex)))
        }
        None => Ok(Cow::Borrowed(self_inc_ex)),
    }
}

/// this is the somewhat hellish logic for deciding:
/// 1. whether we should omit a value at a particular index - returning `None` here
/// 2. and if we are including it, what values of `include` and `exclude` should be passed to it
fn include_or_exclude<'s, 'py>(
    py: Python<'py>,
    index: usize,
    include: &'s IncEx,
    exclude: &'s IncEx,
) -> Option<(Option<&'py PyAny>, Option<&'py PyAny>)> {
    let next_include = match include {
        Some(include) => {
            match include.get(&index) {
                // if the index is in include, based on this, we want to return `Some((next_include, ...))`
                Some(next_include) => next_include
                    .as_ref()
                    .map(|next_include| next_include.clone_ref(py).into_ref(py)),
                // if the index is not in include, this index should be omitted
                None => return None,
            }
        }
        None => None,
    };
    let next_exclude = match exclude {
        Some(exclude) => {
            match exclude.get(&index) {
                Some(next_exclude) => match next_exclude {
                    // if the index is in exclude, and the exclude-value is `Some()`,
                    // we want to return `Some((..., Some(next_exclude))`
                    Some(next_exclude) => Some(next_exclude.clone_ref(py).into_ref(py)),
                    // if the index is in exclude, and the exclude-value is `None`, we want to omit this index
                    None => return None,
                },
                // if the index is not in exclude, based on this, we want to return `Some((..., None))`
                None => None,
            }
        }
        None => None,
    };
    Some((next_include, next_exclude))
}

macro_rules! sequence_serializer_impl {
    (to_python: $type_:ident) => {
        fn to_python(
            &self,
            value: &PyAny,
            format: Option<&str>,
            include: Option<&PyAny>,
            exclude: Option<&PyAny>,
        ) -> PyResult<PyObject> {
            let py = value.py();
            let include = union_inc_ex(py, include, &self.include)?;
            let exclude = union_inc_ex(py, exclude, &self.exclude)?;
            let py_seq: &$type_ = value.cast_as()?;
            let mut items = Vec::with_capacity(py_seq.len());
            let item_serializer = self.item_serializer.as_ref();

            if matches!(item_serializer, CombinedSerializer::Any(_)) && include.is_none() && exclude.is_none() {
                // if we are using the AnySerializer and there is no include/exclude, we can just return the value
                Ok(value.to_object(py))
            } else {
                for (index, element) in py_seq.iter().enumerate() {
                    if let Some((next_include, next_exclude)) = include_or_exclude(py, index, &include, &exclude) {
                        items.push(item_serializer.to_python(element, format, next_include, next_exclude)?);
                    }
                }
                Ok($type_::new(py, items).into_py(py))
            }
        }
    };

    (serde_serialize: $type_:ident) => {
        fn serde_serialize<S: serde::ser::Serializer>(
            &self,
            value: &PyAny,
            serializer: S,
            ob_type_lookup: &ObTypeLookup,
            include: Option<&PyAny>,
            exclude: Option<&PyAny>,
        ) -> Result<S::Ok, S::Error> {
            let py = value.py();
            let include = union_inc_ex(py, include, &self.include).map_err(py_err_se_err)?;
            let exclude = union_inc_ex(py, exclude, &self.exclude).map_err(py_err_se_err)?;
            let item_serializer = self.item_serializer.as_ref();
            let py_seq: &$type_ = value.cast_as().map_err(py_err_se_err)?;

            let mut seq = serializer.serialize_seq(Some(py_seq.len()))?;
            for (index, value) in py_seq.iter().enumerate() {
                if let Some((next_include, next_exclude)) = include_or_exclude(py, index, &include, &exclude) {
                    let item_serialize =
                        PydanticSerializer::new(value, item_serializer, ob_type_lookup, next_include, next_exclude);
                    seq.serialize_element(&item_serialize)?;
                }
            }
            seq.end()
        }
    };
}

build_sequence_serializer!(ListSerializer, "list", &PyList);
impl TypeSerializer for ListSerializer {
    sequence_serializer_impl!(to_python: PyList);
    sequence_serializer_impl!(serde_serialize: PyList);
}

build_sequence_serializer!(TupleSerializer, "tuple", &PyTuple);
impl TypeSerializer for TupleSerializer {
    sequence_serializer_impl!(to_python: PyTuple);

    // just like to_python, but we need to return a list, not a tuple
    fn to_python_json(
        &self,
        value: &PyAny,
        _ob_type_lookup: &ObTypeLookup,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
    ) -> PyResult<PyObject> {
        let py = value.py();
        let include = union_inc_ex(py, include, &self.include)?;
        let exclude = union_inc_ex(py, exclude, &self.exclude)?;
        tuple_to_python_json(value, include, exclude, &self.item_serializer)
    }
    sequence_serializer_impl!(serde_serialize: PyTuple);
}

pub fn tuple_to_python_json(
    value: &PyAny,
    include: Cow<IncEx>,
    exclude: Cow<IncEx>,
    item_serializer: &CombinedSerializer,
) -> PyResult<PyObject> {
    let py = value.py();
    let py_seq: &PyTuple = value.cast_as()?;

    if matches!(item_serializer, CombinedSerializer::Any(_)) && include.is_none() && exclude.is_none() {
        // if we are using the AnySerializer and there is no include/exclude, we can just return the value
        // converted to a list
        Ok(PyList::new(py, py_seq.iter()).into_py(py))
    } else {
        let mut items = Vec::with_capacity(py_seq.len());
        for (index, element) in py_seq.iter().enumerate() {
            if let Some((next_include, next_exclude)) = include_or_exclude(py, index, &include, &exclude) {
                items.push(item_serializer.to_python(element, Some("json"), next_include, next_exclude)?);
            }
        }
        Ok(PyList::new(py, items).into_py(py))
    }
}
