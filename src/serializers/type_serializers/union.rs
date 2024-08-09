use ahash::AHashMap as HashMap;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};
use std::borrow::Cow;

use crate::build_tools::py_schema_err;
use crate::common::discriminator::Discriminator;
use crate::definitions::DefinitionsBuilder;
use crate::lookup_key::LookupKey;
use crate::tools::SchemaDict;
use crate::PydanticSerializationUnexpectedValue;

use super::{
    infer_json_key, infer_serialize, infer_to_python, py_err_se_err, BuildSerializer, CombinedSerializer, Extra,
    SerCheck, TypeSerializer,
};

#[derive(Debug, Clone)]
pub struct UnionSerializer {
    choices: Vec<CombinedSerializer>,
    name: String,
}

impl BuildSerializer for UnionSerializer {
    const EXPECTED_TYPE: &'static str = "union";

    fn build(
        schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let choices: Vec<CombinedSerializer> = schema
            .get_as_req::<Bound<'_, PyList>>(intern!(py, "choices"))?
            .iter()
            .map(|choice| {
                let choice = match choice.downcast::<PyTuple>() {
                    Ok(py_tuple) => py_tuple.get_item(0)?,
                    Err(_) => choice,
                };
                CombinedSerializer::build(choice.downcast()?, config, definitions)
            })
            .collect::<PyResult<Vec<CombinedSerializer>>>()?;

        Self::from_choices(choices)
    }
}

impl UnionSerializer {
    fn from_choices(choices: Vec<CombinedSerializer>) -> PyResult<CombinedSerializer> {
        match choices.len() {
            0 => py_schema_err!("One or more union choices required"),
            1 => Ok(choices.into_iter().next().unwrap()),
            _ => {
                let descr = choices
                    .iter()
                    .map(TypeSerializer::get_name)
                    .collect::<Vec<_>>()
                    .join(", ");
                Ok(Self {
                    choices,
                    name: format!("Union[{descr}]"),
                }
                .into())
            }
        }
    }
}

impl_py_gc_traverse!(UnionSerializer { choices });

impl TypeSerializer for UnionSerializer {
    fn to_python(
        &self,
        value: &Bound<'_, PyAny>,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        // try the serializers in left to right order with error_on fallback=true
        let mut new_extra = extra.clone();
        new_extra.check = SerCheck::Strict;

        for comb_serializer in &self.choices {
            match comb_serializer.to_python(value, include, exclude, &new_extra) {
                Ok(v) => return Ok(v),
                Err(err) => match err.is_instance_of::<PydanticSerializationUnexpectedValue>(value.py()) {
                    true => (),
                    false => return Err(err),
                },
            }
        }
        if self.retry_with_lax_check() {
            new_extra.check = SerCheck::Lax;
            for comb_serializer in &self.choices {
                match comb_serializer.to_python(value, include, exclude, &new_extra) {
                    Ok(v) => return Ok(v),
                    Err(err) => match err.is_instance_of::<PydanticSerializationUnexpectedValue>(value.py()) {
                        true => (),
                        false => return Err(err),
                    },
                }
            }
        }

        extra.warnings.on_fallback_py(self.get_name(), value, extra)?;
        infer_to_python(value, include, exclude, extra)
    }

    fn json_key<'a>(&self, key: &'a Bound<'_, PyAny>, extra: &Extra) -> PyResult<Cow<'a, str>> {
        let mut new_extra = extra.clone();
        new_extra.check = SerCheck::Strict;
        for comb_serializer in &self.choices {
            match comb_serializer.json_key(key, &new_extra) {
                Ok(v) => return Ok(v),
                Err(err) => match err.is_instance_of::<PydanticSerializationUnexpectedValue>(key.py()) {
                    true => (),
                    false => return Err(err),
                },
            }
        }
        if self.retry_with_lax_check() {
            new_extra.check = SerCheck::Lax;
            for comb_serializer in &self.choices {
                match comb_serializer.json_key(key, &new_extra) {
                    Ok(v) => return Ok(v),
                    Err(err) => match err.is_instance_of::<PydanticSerializationUnexpectedValue>(key.py()) {
                        true => (),
                        false => return Err(err),
                    },
                }
            }
        }

        extra.warnings.on_fallback_py(self.get_name(), key, extra)?;
        infer_json_key(key, extra)
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Bound<'_, PyAny>,
        serializer: S,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        let py = value.py();
        let mut new_extra = extra.clone();
        new_extra.check = SerCheck::Strict;
        for comb_serializer in &self.choices {
            match comb_serializer.to_python(value, include, exclude, &new_extra) {
                Ok(v) => return infer_serialize(v.bind(py), serializer, None, None, extra),
                Err(err) => match err.is_instance_of::<PydanticSerializationUnexpectedValue>(py) {
                    true => (),
                    false => return Err(py_err_se_err(err)),
                },
            }
        }
        if self.retry_with_lax_check() {
            new_extra.check = SerCheck::Lax;
            for comb_serializer in &self.choices {
                match comb_serializer.to_python(value, include, exclude, &new_extra) {
                    Ok(v) => return infer_serialize(v.bind(py), serializer, None, None, extra),
                    Err(err) => match err.is_instance_of::<PydanticSerializationUnexpectedValue>(py) {
                        true => (),
                        false => return Err(py_err_se_err(err)),
                    },
                }
            }
        }

        extra.warnings.on_fallback_ser::<S>(self.get_name(), value, extra)?;
        infer_serialize(value, serializer, include, exclude, extra)
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn retry_with_lax_check(&self) -> bool {
        self.choices.iter().any(CombinedSerializer::retry_with_lax_check)
    }
}

#[derive(Debug, Clone)]
pub struct TaggedUnionSerializer {
    discriminator: Discriminator,
    lookup: HashMap<String, CombinedSerializer>,
    choices: Vec<CombinedSerializer>,
    name: String,
}

impl BuildSerializer for TaggedUnionSerializer {
    const EXPECTED_TYPE: &'static str = "tagged-union";

    fn build(
        schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let discriminator = Discriminator::new(py, &schema.get_as_req(intern!(py, "discriminator"))?)?;

        let choice_list: Bound<PyDict> = schema.get_as_req(intern!(py, "choices"))?;
        let mut lookup: HashMap<String, CombinedSerializer> = HashMap::with_capacity(choice_list.len());
        let mut choices: Vec<CombinedSerializer> = Vec::with_capacity(choice_list.len());

        for (choice_key, choice_schema) in choice_list {
            let serializer = CombinedSerializer::build(choice_schema.downcast()?, config, definitions).unwrap();
            choices.push(serializer.clone());
            lookup.insert(choice_key.to_string(), serializer);
        }

        let descr = choices
            .iter()
            .map(TypeSerializer::get_name)
            .collect::<Vec<_>>()
            .join(", ");

        Ok(Self {
            discriminator,
            lookup,
            choices,
            name: format!("TaggedUnion[{descr}]"),
        }
        .into())
    }
}

impl_py_gc_traverse!(TaggedUnionSerializer { discriminator, lookup });

impl TypeSerializer for TaggedUnionSerializer {
    fn to_python(
        &self,
        value: &Bound<'_, PyAny>,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let py = value.py();

        let mut new_extra = extra.clone();
        new_extra.check = SerCheck::Strict;

        if let Some(tag) = self.get_discriminator_value(value) {
            let tag_str = tag.to_string();
            if let Some(serializer) = self.lookup.get(&tag_str) {
                match serializer.to_python(value, include, exclude, &new_extra) {
                    Ok(v) => return Ok(v),
                    Err(err) => match err.is_instance_of::<PydanticSerializationUnexpectedValue>(py) {
                        true => {
                            if self.retry_with_lax_check() {
                                new_extra.check = SerCheck::Lax;
                                return serializer.to_python(value, include, exclude, &new_extra);
                            }
                        }
                        false => return Err(err),
                    },
                }
            }
        }

        let basic_union_ser = UnionSerializer::from_choices(self.choices.clone());
        if let Ok(s) = basic_union_ser {
            return s.to_python(value, include, exclude, extra);
        }

        extra.warnings.on_fallback_py(self.get_name(), value, extra)?;
        infer_to_python(value, include, exclude, extra)
    }

    fn json_key<'a>(&self, key: &'a Bound<'_, PyAny>, extra: &Extra) -> PyResult<Cow<'a, str>> {
        let mut new_extra = extra.clone();
        new_extra.check = SerCheck::Strict;

        if let Some(tag) = self.get_discriminator_value(key) {
            let tag_str = tag.to_string();
            if let Some(serializer) = self.lookup.get(&tag_str) {
                match serializer.json_key(key, &new_extra) {
                    Ok(v) => return Ok(v),
                    Err(_) => {
                        if self.retry_with_lax_check() {
                            new_extra.check = SerCheck::Lax;
                            return serializer.json_key(key, &new_extra);
                        }
                    }
                }
            }
        }

        let basic_union_ser = UnionSerializer::from_choices(self.choices.clone());
        if let Ok(s) = basic_union_ser {
            return s.json_key(key, extra);
        }

        extra.warnings.on_fallback_py(self.get_name(), key, extra)?;
        infer_json_key(key, extra)
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Bound<'_, PyAny>,
        serializer: S,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        let py = value.py();
        let mut new_extra = extra.clone();
        new_extra.check = SerCheck::Strict;

        if let Some(tag) = self.get_discriminator_value(value) {
            let tag_str = tag.to_string();
            if let Some(selected_serializer) = self.lookup.get(&tag_str) {
                match selected_serializer.to_python(value, include, exclude, &new_extra) {
                    Ok(v) => return infer_serialize(v.bind(py), serializer, None, None, extra),
                    Err(_) => {
                        if self.retry_with_lax_check() {
                            new_extra.check = SerCheck::Lax;
                            match selected_serializer.to_python(value, include, exclude, &new_extra) {
                                Ok(v) => return infer_serialize(v.bind(py), serializer, None, None, extra),
                                Err(err) => return Err(py_err_se_err(err)),
                            }
                        }
                    }
                }
            }
        }

        let basic_union_ser = UnionSerializer::from_choices(self.choices.clone());
        if let Ok(s) = basic_union_ser {
            return s.serde_serialize(value, serializer, include, exclude, extra);
        }

        extra.warnings.on_fallback_ser::<S>(self.get_name(), value, extra)?;
        infer_serialize(value, serializer, include, exclude, extra)
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

impl TaggedUnionSerializer {
    fn get_discriminator_value(&self, value: &Bound<'_, PyAny>) -> Option<Py<PyAny>> {
        let py = value.py();
        match &self.discriminator {
            Discriminator::LookupKey(lookup_key) => match lookup_key {
                LookupKey::Simple { py_key, .. } => value.getattr(py_key).ok().map(|obj| obj.to_object(py)),
                _ => None,
            },
            Discriminator::Function(func) => func.call1(py, (value,)).ok().or_else(|| {
                // Try converting object to a dict, might be more compatible with poorly defined callable discriminator
                value
                    .call_method0(intern!(py, "dict"))
                    .and_then(|v| func.call1(py, (v.to_object(py),)))
                    .ok()
            }),
        }
    }
}
