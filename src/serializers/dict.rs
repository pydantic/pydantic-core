use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyString};
use serde::ser::SerializeMap;

use crate::build_tools::SchemaDict;

use super::any::{fallback_serialize, fallback_to_python, AnySerializer};
use super::list_tuple::SchemaIncEx;
use super::shared::{py_err_se_err, BuildSerializer, CombinedSerializer, Extra, SerMode, TypeSerializer};
use super::PydanticSerializer;

#[derive(Debug, Clone)]
pub struct DictSerializer {
    key_serializer: Box<CombinedSerializer>,
    value_serializer: Box<CombinedSerializer>,
    // isize because we look up include exclude via `.hash()` which returns an isize
    inc_ex: SchemaIncEx<isize>,
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
        let inc_ex = match schema.get_as::<&PyDict>(intern!(py, "serialization"))? {
            Some(ser) => {
                let include = ser.get_item(intern!(py, "include"));
                let exclude = ser.get_item(intern!(py, "exclude"));
                SchemaIncEx::new_from_hash(include, exclude)?
            }
            None => SchemaIncEx::default(),
        };
        Ok(Self {
            key_serializer: Box::new(key_serializer),
            value_serializer: Box::new(value_serializer),
            inc_ex,
        }
        .into())
    }
}

impl DictSerializer {
    /// this is the somewhat hellish logic for deciding:
    /// 1. whether we should omit a value at a particular index - returning `Ok(None)` here
    /// 2. and if we are including it, what values of `include` and `exclude` should be passed to it
    fn include_or_exclude<'s, 'py>(
        &'s self,
        key: &PyAny,
        include: Option<&'py PyAny>,
        exclude: Option<&'py PyAny>,
    ) -> PyResult<Option<(Option<&'py PyAny>, Option<&'py PyAny>)>> {
        let hash = key.hash()?;
        self.inc_ex.include_or_exclude(key, hash, include, exclude)
    }
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
                    // note here we use the original key so tuples are represent as '(1, 2)' instead of '[1, 2]'
                    key.str()?.into_py(py)
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
                let value_serializer = self.value_serializer.as_ref();

                let new_dict = PyDict::new(py);
                for (key, value) in py_dict {
                    if let Some((next_include, next_exclude)) = self.include_or_exclude(key, include, exclude)? {
                        let key = self.dict_py_key(key, extra)?;
                        let value = value_serializer.to_python(value, next_include, next_exclude, extra)?;
                        new_dict.set_item(key, value)?;
                    }
                }
                Ok(new_dict.into_py(py))
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
                let mut map = serializer.serialize_map(Some(py_dict.len()))?;
                let key_serializer = self.key_serializer.as_ref();
                let value_serializer = self.value_serializer.as_ref();

                for (key, value) in py_dict {
                    if let Some((next_include, next_exclude)) =
                        self.include_or_exclude(key, include, exclude).map_err(py_err_se_err)?
                    {
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
