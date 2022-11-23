use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use ahash::AHashMap;
use serde::ser::SerializeMap;

use crate::build_tools::{py_error_type, schema_or_config, SchemaDict};
use crate::serializers::any::SerializeInfer;

use super::any::{fallback_serialize, fallback_to_python, json_key};
use super::shared::{py_err_se_err, BuildSerializer, CombinedSerializer, Extra, TypeSerializer};
use super::PydanticSerializer;

#[derive(Debug, Clone)]
struct TypedDictField {
    index: usize,
    key_py: Py<PyString>,
    alias: Option<String>,
    alias_py: Option<Py<PyString>>,
    serializer: CombinedSerializer,
    include: bool,
}

#[derive(Debug, Clone)]
pub struct TypedDictSerializer {
    fields: AHashMap<String, TypedDictField>,
    include_extra: bool,
}

impl BuildSerializer for TypedDictSerializer {
    const EXPECTED_TYPE: &'static str = "typed-dict";

    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        let py = schema.py();

        let extra_behavior = schema_or_config::<&str>(
            schema,
            config,
            intern!(py, "extra_behavior"),
            intern!(py, "typed_dict_extra_behavior"),
        )?;

        let include_extra = extra_behavior == Some("allow");

        let fields_dict: &PyDict = schema.get_as_req(intern!(py, "fields"))?;
        let mut fields: AHashMap<String, TypedDictField> = AHashMap::with_capacity(fields_dict.len());

        for (index, (key, value)) in fields_dict.iter().enumerate() {
            let key: String = key.extract()?;
            let field_info: &PyDict = value.cast_as()?;

            let schema = field_info.get_as_req(intern!(py, "schema"))?;

            let serializer =
                CombinedSerializer::build(schema, config).map_err(|e| py_error_type!("Field `{}`:\n  {}", key, e))?;

            let (alias, alias_py) = match field_info.get_as::<&PyString>(intern!(py, "serialization_alias"))? {
                Some(alias_py) => {
                    let alias: String = alias_py.extract()?;
                    (Some(alias), Some(alias_py.into_py(py)))
                }
                None => (None, None),
            };

            let key_py = PyString::intern(py, &key).into_py(py);
            fields.insert(
                key,
                TypedDictField {
                    index,
                    key_py,
                    alias,
                    alias_py,
                    serializer,
                    include: field_info.get_as(intern!(py, "serialization_include"))?.unwrap_or(true),
                },
            );
        }

        Ok(Self { fields, include_extra }.into())
    }
}

impl TypeSerializer for TypedDictSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        // TODO include and exclude

        let py = value.py();
        match value.cast_as::<PyDict>() {
            Ok(py_dict) => {
                let mut new_values: Vec<Option<(PyObject, PyObject)>> = (0..self.fields.len()).map(|_| None).collect();

                for (key, value) in py_dict {
                    if let Ok(key_py_str) = key.cast_as::<PyString>() {
                        if let Some(field) = self.fields.get(key_py_str.to_str()?) {
                            if field.include {
                                let value = field.serializer.to_python(value, None, None, extra)?;
                                let key_py = if let Some(ref alias_py) = field.alias_py {
                                    alias_py.as_ref(py)
                                } else {
                                    field.key_py.as_ref(py)
                                };
                                new_values[field.index] = Some((key_py.into_py(py), value));
                            }
                            continue;
                        }
                    }
                    if self.include_extra {
                        let value = fallback_to_python(value, extra)?;
                        new_values.push(Some((key.into_py(py), value)));
                    }
                }
                // TODO: would it be faster here to use `from_sequence`?
                let new_dict = PyDict::new(py);
                for (key, value) in new_values.into_iter().flatten() {
                    new_dict.set_item(key, value)?;
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
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        // TODO include and exclude

        let py = value.py();
        match value.cast_as::<PyDict>() {
            Ok(py_dict) => {
                let mut new_values: Vec<Option<(String, Option<&TypedDictField>, PyObject)>> =
                    (0..self.fields.len()).map(|_| None).collect();

                for (key, value) in py_dict {
                    if let Ok(key_py_str) = key.cast_as::<PyString>() {
                        let key_str = key_py_str.to_str().map_err(py_err_se_err)?;
                        if let Some(field) = self.fields.get(key_str) {
                            if field.include {
                                let output_key = if let Some(ref alias) = field.alias {
                                    alias.clone()
                                } else {
                                    key_str.to_string()
                                };
                                new_values[field.index] = Some((output_key, Some(field), value.into_py(py)));
                            }
                            continue;
                        }
                    }
                    if self.include_extra {
                        let output_key = json_key(key, extra).map_err(py_err_se_err)?.to_string();
                        new_values.push(Some((output_key, None, value.into_py(py))));
                    }
                }
                let mut map = serializer.serialize_map(Some(py_dict.len()))?;
                for (key, op_field, value) in new_values.into_iter().flatten() {
                    if let Some(field) = op_field {
                        let s = PydanticSerializer::new(value.as_ref(py), &field.serializer, None, None, extra);
                        map.serialize_entry(&key, &s)
                    } else {
                        let s = SerializeInfer::new(value.as_ref(py), extra.ob_type_lookup);
                        map.serialize_entry(&key, &s)
                    }?;
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
