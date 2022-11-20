use pyo3::exceptions::PyTypeError;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PySet, PyString};
use serde::ser::SerializeMap;

use crate::build_tools::SchemaDict;

use super::any::{fallback_serialize, fallback_to_python, AnySerializer};
use super::shared::{py_err_se_err, BuildSerializer, CombinedSerializer, Extra, SerMode, TypeSerializer};
use super::PydanticSerializer;

// TODO instead of using a plain of PyDict, we might be able to use a custom type like some kind of aHashmap
pub fn to_inc_ex(value: Option<&PyAny>) -> PyResult<Option<&PyDict>> {
    match value {
        Some(value) => {
            if let Ok(py_set) = value.cast_as::<PySet>() {
                let py = value.py();
                let py_dict = PyDict::new(py);
                for item in py_set.iter() {
                    py_dict.set_item(item, py.None())?;
                }
                Ok(Some(py_dict))
            } else if let Ok(py_dict) = value.cast_as::<PyDict>() {
                Ok(Some(py_dict))
            } else {
                Err(PyTypeError::new_err(
                    "`include` and `exclude` inputs must be sets or dicts.",
                ))
            }
        }
        None => Ok(None),
    }
}

#[derive(Debug, Clone)]
pub struct DictSerializer {
    key_serializer: Box<CombinedSerializer>,
    value_serializer: Box<CombinedSerializer>,
    include: Option<Py<PyDict>>,
    exclude: Option<Py<PyDict>>,
}

impl BuildSerializer for DictSerializer {
    const EXPECTED_TYPE: &'static str = "dict";

    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let key_serializer = match schema.get_as::<&PyDict>(intern!(py, "keys_schema"))? {
            Some(items_schema) => CombinedSerializer::build(items_schema, config)?,
            None => AnySerializer::build(schema, config)?,
        };
        let value_serializer = match schema.get_as::<&PyDict>(intern!(py, "values_schema"))? {
            Some(items_schema) => CombinedSerializer::build(items_schema, config)?,
            None => AnySerializer::build(schema, config)?,
        };
        let (include, exclude) = match schema.get_as::<&PyDict>(intern!(py, "serialization"))? {
            Some(ser) => {
                let include = to_inc_ex(ser.get_item(intern!(py, "include")))?;
                let exclude = to_inc_ex(ser.get_item(intern!(py, "exclude")))?;
                (include, exclude)
            }
            None => (None, None),
        };
        Ok(Self {
            key_serializer: Box::new(key_serializer),
            value_serializer: Box::new(value_serializer),
            include: include.map(|x| x.into_py(py)),
            exclude: exclude.map(|x| x.into_py(py)),
        }
        .into())
    }
}

/// Combine the serialization time include/exclude with the include/exclude when creating the serializer
/// **NOTE:** we merge with union for both include and exclude, this is a change from V1 where we did,
/// union for exclude and intersection for include
#[cfg_attr(debug_assertions, derive(Debug))]
struct IncEx<'py>(Option<&'py PyDict>, &'py Option<Py<PyDict>>);

impl<'py> IncEx<'py> {
    fn new(ser_time: Option<&'py PyAny>, self_value: &'py Option<Py<PyDict>>) -> PyResult<Option<Self>> {
        let v0 = to_inc_ex(ser_time)?;
        if v0.is_none() && self_value.is_none() {
            Ok(None)
        } else {
            Ok(Some(Self(v0, self_value)))
        }
    }

    fn get(&self, key: &'py PyAny) -> Option<&'py PyAny> {
        let v0 = match self.0 {
            Some(d0) => d0.get_item(key),
            None => None,
        };
        match v0 {
            Some(v0) => Some(v0),
            None => match self.1 {
                Some(v1) => v1.as_ref(key.py()).get_item(key),
                None => None,
            },
        }
    }
}

/// this is the somewhat hellish logic for deciding:
/// 1. whether we should omit a value at a particular index - returning `None` here
/// 2. and if we are including it, what values of `include` and `exclude` should be passed to it
fn include_or_exclude<'py>(
    key: &'py PyAny,
    include: &'py Option<IncEx>,
    exclude: &'py Option<IncEx>,
) -> Option<(Option<&'py PyAny>, Option<&'py PyAny>)> {
    let next_include = match include {
        Some(include) => {
            match include.get(key) {
                // if the key is in include, based on this, we want to return `Some((next_include, ...))`
                Some(next_include) => Some(next_include),
                // if the key is not in include, this pair should be omitted
                None => return None,
            }
        }
        // no include, so we want to return `Some((None, ...))`
        None => None,
    };
    let next_exclude = match exclude {
        Some(exclude) => {
            match exclude.get(key) {
                Some(next_exclude) => match next_exclude.is_none() {
                    // if the index is in exclude, and the exclude-value is `None`, we want to omit this index
                    true => return None,
                    // if the index is in exclude, and the exclude-value is not `None`,
                    // we want to return `Some((..., Some(next_exclude))`
                    false => Some(next_exclude),
                },
                // if the index is not in exclude, based on this, we want to return `Some((..., None))`
                None => None,
            }
        }
        // no exclude, so we want to return `Some((..., None))`
        None => None,
    };
    Some((next_include, next_exclude))
}

impl DictSerializer {
    fn dict_py_key(&self, key: &PyAny, extra: &Extra) -> PyResult<PyObject> {
        let py = key.py();
        let raw_key: PyObject = self.key_serializer.to_python(key, None, None, extra)?;
        let key = match extra.mode {
            SerMode::Json => {
                let k = raw_key.as_ref(py);
                if k.is_instance_of::<PyString>().unwrap_or(false) {
                    raw_key
                } else if let Ok(py_bool) = k.cast_as::<PyBool>() {
                    if py_bool.is_true() {
                        intern!(py, "true")
                    } else {
                        intern!(py, "false")
                    }
                    .into_py(py)
                } else {
                    k.str()?.into_py(py)
                }
            }
            _ => raw_key,
        };
        Ok(key)
    }
}

impl TypeSerializer for DictSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let py = value.py();
        match value.cast_as::<PyDict>() {
            Ok(py_dict) => {
                let include = IncEx::new(include, &self.include)?;
                let exclude = IncEx::new(exclude, &self.exclude)?;

                let key_serializer = self.key_serializer.as_ref();
                let value_serializer = self.value_serializer.as_ref();

                if key_serializer.is_any()
                    && value_serializer.is_any()
                    && include.is_none()
                    && exclude.is_none()
                    && !extra.mode.is_json()
                {
                    // if we are using AnySerializers and there is no include/exclude, we can just return the value
                    Ok(py_dict.into_py(py))
                } else {
                    let new_dict = PyDict::new(py);
                    for (key, value) in py_dict {
                        if let Some((next_include, next_exclude)) = include_or_exclude(key, &include, &exclude) {
                            let key = self.dict_py_key(key, extra)?;
                            let value = value_serializer.to_python(value, next_include, next_exclude, extra)?;
                            new_dict.set_item(key, value)?;
                        }
                    }
                    Ok(new_dict.into_py(py))
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
        match value.cast_as::<PyDict>() {
            Ok(py_dict) => {
                let include = IncEx::new(include, &self.include).map_err(py_err_se_err)?;
                let exclude = IncEx::new(exclude, &self.exclude).map_err(py_err_se_err)?;

                let mut map = serializer.serialize_map(Some(py_dict.len()))?;
                let key_serializer = self.key_serializer.as_ref();
                let value_serializer = self.value_serializer.as_ref();

                for (key, value) in py_dict {
                    if let Some((next_include, next_exclude)) = include_or_exclude(key, &include, &exclude) {
                        let key = key_serializer.json_key(key, extra).map_err(py_err_se_err)?;
                        let value_serialize =
                            PydanticSerializer::new(value, value_serializer, next_include, next_exclude, extra);
                        map.serialize_entry(&key, &value_serialize)?;
                    }
                }
                map.end()
            }
            Err(_) => {
                extra.warnings.fallback_filtering(Self::EXPECTED_TYPE, value);
                fallback_serialize(value, serializer, extra.ob_type_lookup)
            }
        }
    }
}
