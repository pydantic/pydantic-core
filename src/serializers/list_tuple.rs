use std::borrow::Cow;
use std::hash::BuildHasherDefault;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PySet, PyTuple};

use nohash_hasher::IntMap;
use pyo3::exceptions::PyTypeError;
use serde::ser::SerializeSeq;

use crate::build_tools::SchemaDict;

use super::any::{fallback_serialize, fallback_to_python, fallback_to_python_json, AnySerializer};
use super::{
    build_serializer, py_err_se_err, BuildSerializer, CombinedSerializer, Extra, PydanticSerializer, TypeSerializer,
};

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
            } else if let Ok(py_dict) = value.cast_as::<PyDict>() {
                let py = value.py();
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
            } else {
                Err(PyTypeError::new_err(
                    "`include` and `exclude` inputs must be sets or dicts.",
                ))
            }
        }
        None => Ok(None),
    }
}

macro_rules! build_serializer {
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
                let (include, exclude) = match schema.get_as::<&PyDict>(pyo3::intern!(py, "serialization"))? {
                    Some(ser) => {
                        let include = to_inc_ex(ser.get_item(pyo3::intern!(py, "include")))?;
                        let exclude = to_inc_ex(ser.get_item(pyo3::intern!(py, "exclude")))?;
                        (include, exclude)
                    }
                    None => (None, None),
                };
                Ok(Self {
                    item_serializer: Box::new(item_serializer),
                    include,
                    exclude,
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

build_serializer!(ListSerializer, "list", &PyList);

impl TypeSerializer for ListSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let py = value.py();
        match value.cast_as::<PyList>() {
            Ok(py_list) => {
                let include = union_inc_ex(py, include, &self.include)?;
                let exclude = union_inc_ex(py, exclude, &self.exclude)?;

                let mut items = Vec::with_capacity(py_list.len());
                let item_serializer = self.item_serializer.as_ref();

                if matches!(item_serializer, CombinedSerializer::Any(_)) && include.is_none() && exclude.is_none() {
                    // if we are using the AnySerializer and there is no include/exclude, we can just return the value
                    Ok(py_list.to_object(py))
                } else {
                    for (index, element) in py_list.iter().enumerate() {
                        if let Some((next_include, next_exclude)) = include_or_exclude(py, index, &include, &exclude) {
                            items.push(item_serializer.to_python(element, next_include, next_exclude, extra)?);
                        }
                    }
                    Ok(items.into_py(py))
                }
            }
            // since there's no `to_python_json` method, this method is called, thus we need to handle format='json'
            // correctly here
            Err(_) => {
                extra.warnings.fallback(Self::EXPECTED_TYPE, value);
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
                let py = value.py();
                let include = union_inc_ex(py, include, &self.include).map_err(py_err_se_err)?;
                let exclude = union_inc_ex(py, exclude, &self.exclude).map_err(py_err_se_err)?;

                let mut seq = serializer.serialize_seq(Some(py_list.len()))?;
                let item_serializer = self.item_serializer.as_ref();

                for (index, value) in py_list.iter().enumerate() {
                    if let Some((next_include, next_exclude)) = include_or_exclude(py, index, &include, &exclude) {
                        let item_serialize =
                            PydanticSerializer::new(value, item_serializer, next_include, next_exclude, extra);
                        seq.serialize_element(&item_serialize)?;
                    }
                }
                seq.end()
            }
            Err(_) => fallback_serialize(value, serializer, extra.ob_type_lookup),
        }
    }
}

build_serializer!(TupleSerializer, "tuple", &PyTuple);

impl TypeSerializer for TupleSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let py = value.py();
        match value.cast_as::<PyTuple>() {
            Ok(py_tuple) => {
                let include = union_inc_ex(py, include, &self.include)?;
                let exclude = union_inc_ex(py, exclude, &self.exclude)?;

                let mut items = Vec::with_capacity(py_tuple.len());
                let item_serializer = self.item_serializer.as_ref();

                if matches!(item_serializer, CombinedSerializer::Any(_)) && include.is_none() && exclude.is_none() {
                    // if we are using the AnySerializer and there is no include/exclude, we can just return the value
                    Ok(py_tuple.to_object(py))
                } else {
                    for (index, element) in py_tuple.iter().enumerate() {
                        if let Some((next_include, next_exclude)) = include_or_exclude(py, index, &include, &exclude) {
                            items.push(item_serializer.to_python(element, next_include, next_exclude, extra)?);
                        }
                    }
                    Ok(PyTuple::new(py, items).into_py(py))
                }
            }
            Err(_) => Ok(value.into_py(py)),
        }
    }

    // just like to_python, but we need to return a list, not a tuple
    fn to_python_json(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let py = value.py();
        let include = union_inc_ex(py, include, &self.include)?;
        let exclude = union_inc_ex(py, exclude, &self.exclude)?;

        match value.cast_as::<PyTuple>() {
            Ok(py_tuple) => {
                let item_serializer = self.item_serializer.as_ref();

                if matches!(item_serializer, CombinedSerializer::Any(_)) && include.is_none() && exclude.is_none() {
                    // if we are using the AnySerializer and there is no include/exclude, we can just return the value
                    // converted to a list
                    Ok(PyList::new(py, py_tuple.iter()).into_py(py))
                } else {
                    let mut items = Vec::with_capacity(py_tuple.len());
                    for (index, element) in py_tuple.iter().enumerate() {
                        if let Some((next_include, next_exclude)) = include_or_exclude(py, index, &include, &exclude) {
                            items.push(item_serializer.to_python(element, next_include, next_exclude, extra)?);
                        }
                    }
                    Ok(PyList::new(py, items).into_py(py))
                }
            }
            Err(_) => fallback_to_python_json(value, extra.ob_type_lookup),
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
                let py = value.py();
                let include = union_inc_ex(py, include, &self.include).map_err(py_err_se_err)?;
                let exclude = union_inc_ex(py, exclude, &self.exclude).map_err(py_err_se_err)?;

                let py_tuple: &PyTuple = py_tuple.cast_as().map_err(py_err_se_err)?;
                let item_serializer = self.item_serializer.as_ref();

                let mut seq = serializer.serialize_seq(Some(py_tuple.len()))?;
                for (index, value) in py_tuple.iter().enumerate() {
                    if let Some((next_include, next_exclude)) = include_or_exclude(py, index, &include, &exclude) {
                        let item_serialize =
                            PydanticSerializer::new(value, item_serializer, next_include, next_exclude, extra);
                        seq.serialize_element(&item_serialize)?;
                    }
                }
                seq.end()
            }
            Err(_) => fallback_serialize(value, serializer, extra.ob_type_lookup),
        }
    }
}
