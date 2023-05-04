use std::borrow::Cow;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use ahash::AHashMap;
use serde::ser::SerializeMap;

use crate::PydanticSerializationUnexpectedValue;

use super::computed_fields::ComputedFields;
use super::errors::py_err_se_err;
use super::extra::Extra;
use super::filter::SchemaFilter;
use super::infer::{infer_json_key, infer_serialize, infer_to_python, SerializeInfer};
use super::shared::PydanticSerializer;
use super::shared::{CombinedSerializer, TypeSerializer};

/// representation of a field for serialization
#[derive(Debug, Clone)]
pub(super) struct SerField {
    pub key_py: Py<PyString>,
    pub alias: Option<String>,
    pub alias_py: Option<Py<PyString>>,
    // None serializer means exclude
    pub serializer: Option<CombinedSerializer>,
    pub required: bool,
}

impl SerField {
    pub fn new(
        py: Python,
        key_py: Py<PyString>,
        alias: Option<String>,
        serializer: Option<CombinedSerializer>,
        required: bool,
    ) -> Self {
        let alias_py = alias.as_ref().map(|alias| PyString::new(py, alias.as_str()).into());
        Self {
            key_py,
            alias,
            alias_py,
            serializer,
            required,
        }
    }

    pub fn get_key_py<'py>(&'py self, py: Python<'py>, extra: &Extra) -> &'py PyAny {
        if extra.by_alias {
            if let Some(ref alias_py) = self.alias_py {
                return alias_py.as_ref(py);
            }
        }
        self.key_py.as_ref(py)
    }

    pub fn get_key_json<'a>(&'a self, key_str: &'a str, extra: &Extra) -> Cow<'a, str> {
        if extra.by_alias {
            if let Some(ref alias) = self.alias {
                return Cow::Borrowed(alias.as_str());
            }
        }
        Cow::Borrowed(key_str)
    }

    pub fn to_python(
        &self,
        output_dict: &PyDict,
        value: &PyAny,
        next_include: Option<&PyAny>,
        next_exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<()> {
        if let Some(ref serializer) = self.serializer {
            if !exclude_default(value, extra, serializer)? {
                let value = serializer.to_python(value, next_include, next_exclude, extra)?;
                let output_key = self.get_key_py(output_dict.py(), extra);
                output_dict.set_item(output_key, value)?;
            }
        }
        Ok(())
    }
}

fn exclude_default(value: &PyAny, extra: &Extra, serializer: &CombinedSerializer) -> PyResult<bool> {
    if extra.exclude_defaults {
        if let Some(default) = serializer.get_default(value.py())? {
            if value.eq(default)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

#[derive(Debug, Clone)]
pub struct GeneralFieldsSerializer {
    fields: AHashMap<String, SerField>,
    computed_fields: Option<ComputedFields>,
    include_extra: bool,
    // isize because we look up filter via `.hash()` which returns an isize
    filter: SchemaFilter<isize>,
    required_fields: usize,
}

impl GeneralFieldsSerializer {
    pub(super) fn new(
        fields: AHashMap<String, SerField>,
        include_extra: bool,
        computed_fields: Option<ComputedFields>,
    ) -> Self {
        let required_fields = fields.values().filter(|f| f.required).count();
        Self {
            fields,
            include_extra,
            filter: SchemaFilter::default(),
            computed_fields,
            required_fields,
        }
    }

    pub fn n_computed_fields(&self) -> usize {
        match self.computed_fields {
            None => 0,
            Some(ref computed_fields) => computed_fields.len(),
        }
    }
}

impl TypeSerializer for GeneralFieldsSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let py = value.py();
        // If there is already a model registered (from a dataclass, BaseModel)
        // then do not touch it
        // If there is no model, we (a TypedDict) are the model
        let td_extra = Extra {
            model: extra.model.map_or_else(|| Some(value), Some),
            ..*extra
        };
        let (main_dict, extra_dict) = if let Ok(main_dict) = value.downcast::<PyDict>() {
            (main_dict, None)
        } else if let Ok((main_dict, extra_dict)) = value.extract::<(&PyDict, &PyDict)>() {
            (main_dict, Some(extra_dict))
        } else {
            td_extra.warnings.on_fallback_py(self.get_name(), value, &td_extra)?;
            return infer_to_python(value, include, exclude, &td_extra);
        };

        // NOTE! we maintain the order of the input dict assuming that's right
        let output_dict = PyDict::new(py);
        let mut used_req_fields: usize = 0;

        for (key, value) in main_dict {
            if extra.exclude_none && value.is_none() {
                continue;
            }
            if let Some((next_include, next_exclude)) = self.filter.key_filter(key, include, exclude)? {
                let extra = Extra {
                    field_name: Some(key.extract()?),
                    ..td_extra
                };
                if let Ok(key_py_str) = key.downcast::<PyString>() {
                    let key_str = key_py_str.to_str()?;
                    if let Some(field) = self.fields.get(key_str) {
                        field.to_python(output_dict, value, next_include, next_exclude, &extra)?;

                        if field.required {
                            used_req_fields += 1;
                        }
                        continue;
                    }
                }
                if self.include_extra {
                    // TODO test this
                    let value = infer_to_python(value, next_include, next_exclude, &extra)?;
                    output_dict.set_item(key, value)?;
                } else if extra.check.enabled() {
                    return Err(PydanticSerializationUnexpectedValue::new_err(None));
                }
            }
        }
        if td_extra.check.enabled() && self.required_fields != used_req_fields {
            return Err(PydanticSerializationUnexpectedValue::new_err(None));
        }
        // this is used to include `__pydantic_extra__` in serialization on models
        if let Some(extra_dict) = extra_dict {
            for (key, value) in extra_dict.iter() {
                if let Some((next_include, next_exclude)) = self.filter.key_filter(key, include, exclude)? {
                    let value = infer_to_python(value, next_include, next_exclude, &td_extra)?;
                    output_dict.set_item(key, value)?;
                }
            }
        }
        if let Some(ref computed_fields) = self.computed_fields {
            if let Some(model) = td_extra.model {
                computed_fields.to_python(model, output_dict, &self.filter, include, exclude, &td_extra)?;
            }
        }
        Ok(output_dict.into_py(py))
    }

    fn json_key<'py>(&self, key: &'py PyAny, extra: &Extra) -> PyResult<Cow<'py, str>> {
        self._invalid_as_json_key(key, extra, "fields")
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        match value.downcast::<PyDict>() {
            Ok(py_dict) => {
                // If there is already a model registered (from a dataclass, BaseModel)
                // then do not touch it
                // If there is no model, we (a TypedDict) are the model
                let td_extra = Extra {
                    model: extra.model.map_or_else(|| Some(value), Some),
                    ..*extra
                };
                let expected_len = match self.include_extra {
                    true => py_dict.len(),
                    false => self.fields.len() + self.n_computed_fields(),
                };
                // NOTE! As above, we maintain the order of the input dict assuming that's right
                // we don't both with `used_fields` here because on unions, `to_python(..., mode='json')` is used
                let mut map = serializer.serialize_map(Some(expected_len))?;

                for (key, value) in py_dict {
                    let extra = Extra {
                        field_name: Some(key.extract().map_err(py_err_se_err)?),
                        ..td_extra
                    };
                    if extra.exclude_none && value.is_none() {
                        continue;
                    }
                    if let Some((next_include, next_exclude)) =
                        self.filter.key_filter(key, include, exclude).map_err(py_err_se_err)?
                    {
                        if let Ok(key_py_str) = key.downcast::<PyString>() {
                            let key_str = key_py_str.to_str().map_err(py_err_se_err)?;
                            if let Some(field) = self.fields.get(key_str) {
                                if let Some(ref serializer) = field.serializer {
                                    if !exclude_default(value, &extra, serializer).map_err(py_err_se_err)? {
                                        let s = PydanticSerializer::new(
                                            value,
                                            serializer,
                                            next_include,
                                            next_exclude,
                                            &extra,
                                        );
                                        let output_key = field.get_key_json(key_str, &extra);
                                        map.serialize_entry(&output_key, &s)?;
                                    }
                                }
                                continue;
                            }
                        }
                        if self.include_extra {
                            let s = SerializeInfer::new(value, include, exclude, &extra);
                            let output_key = infer_json_key(key, &extra).map_err(py_err_se_err)?;
                            map.serialize_entry(&output_key, &s)?
                        }
                    }
                }
                if let Some(ref computed_fields) = self.computed_fields {
                    if let Some(model) = td_extra.model {
                        computed_fields.serde_serialize::<S>(model, &mut map, &self.filter, include, exclude, extra)?;
                    }
                }
                map.end()
            }
            Err(_) => {
                extra.warnings.on_fallback_ser::<S>(self.get_name(), value, extra)?;
                infer_serialize(value, serializer, include, exclude, extra)
            }
        }
    }

    fn get_name(&self) -> &str {
        "fields"
    }
}
