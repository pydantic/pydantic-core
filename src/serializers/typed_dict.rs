use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};
use std::borrow::Cow;

use ahash::AHashMap;
use serde::ser::SerializeMap;

use crate::build_tools::{py_error_type, schema_or_config, SchemaDict};
use crate::serializers::any::SerializeInfer;

use super::any::{fallback_serialize, fallback_to_python, json_key};
use super::include_exclude::SchemaIncEx;
use super::shared::{py_err_se_err, BuildSerializer, CombinedSerializer, Extra, TypeSerializer};
use super::PydanticSerializer;

#[derive(Debug, Clone)]
struct TypedDictField {
    index: usize,
    key_py: Py<PyString>,
    alias: Option<String>,
    alias_py: Option<Py<PyString>>,
    serializer: CombinedSerializer,
}

#[derive(Debug, Clone)]
pub struct TypedDictSerializer {
    fields: AHashMap<String, TypedDictField>,
    include_extra: bool,
    // isize because we look up include exclude via `.hash()` which returns an isize
    inc_ex: SchemaIncEx<isize>,
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
        let mut exclude: Vec<Py<PyString>> = Vec::with_capacity(fields_dict.len());

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

            let key_py: Py<PyString> = PyString::intern(py, &key).into_py(py);

            if field_info.get_as(intern!(py, "serialization_exclude"))? == Some(true) {
                exclude.push(key_py.clone_ref(py));
            }
            fields.insert(
                key,
                TypedDictField {
                    index,
                    key_py,
                    alias,
                    alias_py,
                    serializer,
                },
            );
        }

        let inc_ex = SchemaIncEx::from_vec_hash(py, exclude)?;

        Ok(Self {
            fields,
            include_extra,
            inc_ex,
        }
        .into())
    }
}

enum ValueSerializer<'py> {
    Infer(SerializeInfer<'py>),
    Field(PydanticSerializer<'py>),
}

impl TypeSerializer for TypedDictSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        // TODO include and exclude

        let py = value.py();
        match value.cast_as::<PyDict>() {
            Ok(py_dict) => {
                // we build a temporary vec, then iterate over it so the order of the output dict matches
                // the order of the fields in the schema
                let mut new_items: Vec<Option<(&PyAny, PyObject)>> = (0..self.fields.len()).map(|_| None).collect();

                for (key, value) in py_dict {
                    if let Some((next_include, next_exclude)) =
                        self.inc_ex.include_or_exclude_key(key, include, exclude)?
                    {
                        if let Ok(key_py_str) = key.cast_as::<PyString>() {
                            if let Some(field) = self.fields.get(key_py_str.to_str()?) {
                                let value = field.serializer.to_python(value, next_include, next_exclude, extra)?;
                                let key_py = if let Some(ref alias_py) = field.alias_py {
                                    alias_py.as_ref(py)
                                } else {
                                    field.key_py.as_ref(py)
                                };
                                new_items[field.index] = Some((key_py, value));
                                continue;
                            }
                        }
                        if self.include_extra {
                            let value = fallback_to_python(value, extra)?;
                            new_items.push(Some((key, value)));
                        }
                    }
                }
                // TODO: would it be faster here to use `from_sequence`?
                let new_dict = PyDict::new(py);
                for (key, value) in new_items.into_iter().flatten() {
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
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        // TODO include and exclude

        match value.cast_as::<PyDict>() {
            Ok(py_dict) => {
                let expected_len = match self.include_extra {
                    true => py_dict.len(),
                    false => self.fields.len(),
                };
                // as above, we build a temporary vec, then iterate over it so the order of the output dict matches
                // the order of the fields in the schema
                let mut ser_items: Vec<Option<(Cow<str>, ValueSerializer)>> = (0..expected_len).map(|_| None).collect();

                for (key, value) in py_dict {
                    if let Some((next_include, next_exclude)) = self
                        .inc_ex
                        .include_or_exclude_key(key, include, exclude)
                        .map_err(py_err_se_err)?
                    {
                        if let Ok(key_py_str) = key.cast_as::<PyString>() {
                            let key_str = key_py_str.to_str().map_err(py_err_se_err)?;
                            if let Some(field) = self.fields.get(key_str) {
                                let output_key = if let Some(ref alias) = field.alias {
                                    Cow::Borrowed(alias.as_str())
                                } else {
                                    Cow::Borrowed(key_str)
                                };
                                let s = PydanticSerializer::new(
                                    value,
                                    &field.serializer,
                                    next_include,
                                    next_exclude,
                                    extra,
                                );
                                ser_items[field.index] = Some((output_key, ValueSerializer::Field(s)));
                                continue;
                            }
                        }
                        if self.include_extra {
                            let s = SerializeInfer::new(value, extra.ob_type_lookup);
                            let output_key = json_key(key, extra).map_err(py_err_se_err)?;
                            ser_items.push(Some((output_key, ValueSerializer::Infer(s))));
                        }
                    }
                }
                let mut map = serializer.serialize_map(Some(ser_items.len()))?;
                for (key, serializer) in ser_items.into_iter().flatten() {
                    match serializer {
                        ValueSerializer::Infer(s) => map.serialize_entry(&key, &s)?,
                        ValueSerializer::Field(s) => map.serialize_entry(&key, &s)?,
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
