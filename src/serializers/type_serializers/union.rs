use ahash::AHashMap as HashMap;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::sync::Arc;

use crate::build_tools::py_schema_err;
use crate::common::union::{Discriminator, SMALL_UNION_THRESHOLD};
use crate::definitions::DefinitionsBuilder;
use crate::serializers::PydanticSerializationUnexpectedValue;
use crate::tools::{truncate_safe_repr, SchemaDict};

use super::{
    infer_json_key, infer_serialize, infer_to_python, BuildSerializer, CombinedSerializer, Extra, SerCheck,
    TypeSerializer,
};

#[derive(Debug)]
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

fn union_serialize<S, R>(
    // if this returns `Ok(v)`, we picked a union variant to serialize, where
    // `S` is intermediate state which can be passed on to the finalizer
    mut selector: impl FnMut(&CombinedSerializer, &Extra) -> PyResult<S>,
    // if called with `Some(v)`, we have intermediate state to finish
    // if `None`, we need to just go to fallback
    finalizer: impl FnOnce(Option<S>) -> R,
    extra: &Extra,
    choices: &[CombinedSerializer],
    retry_with_lax_check: bool,
) -> PyResult<R> {
    // try the serializers in left to right order with error_on fallback=true
    let mut new_extra = extra.clone();
    new_extra.check = SerCheck::Strict;
    let mut errors: SmallVec<[PyErr; SMALL_UNION_THRESHOLD]> = SmallVec::new();

    for comb_serializer in choices {
        match selector(comb_serializer, &new_extra) {
            Ok(v) => return Ok(finalizer(Some(v))),
            Err(err) => errors.push(err),
        }
    }

    // If extra.check is SerCheck::Strict, we're in a nested union
    if extra.check != SerCheck::Strict && retry_with_lax_check {
        new_extra.check = SerCheck::Lax;
        for comb_serializer in choices {
            if let Ok(v) = selector(comb_serializer, &new_extra) {
                return Ok(finalizer(Some(v)));
            }
        }
    }

    // If extra.check is SerCheck::None, we're in a top-level union. We should thus raise the warnings
    if extra.check == SerCheck::None {
        for err in &errors {
            extra.warnings.custom_warning(err.to_string());
        }
    }
    // Otherwise, if we've encountered errors, return them to the parent union, which should take
    // care of the formatting for us
    else if !errors.is_empty() {
        let message = errors.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
        return Err(PydanticSerializationUnexpectedValue::new_err(Some(message)));
    }

    Ok(finalizer(None))
}

fn tagged_union_serialize<S>(
    discriminator_value: Option<Py<PyAny>>,
    lookup: &HashMap<String, usize>,
    // if this returns `Ok(v)`, we picked a union variant to serialize, where
    // `S` is intermediate state which can be passed on to the finalizer
    mut selector: impl FnMut(&CombinedSerializer, &Extra) -> PyResult<S>,
    extra: &Extra,
    choices: &Vec<CombinedSerializer>,
    retry_with_lax_check: bool,
) -> Option<S> {
    let mut new_extra = extra.clone();
    new_extra.check = SerCheck::Strict;

    if let Some(tag) = discriminator_value {
        let tag_str = tag.to_string();
        if let Some(&serializer_index) = lookup.get(&tag_str) {
            let selected_serializer = &choices[serializer_index];

            match selector(&selected_serializer, &new_extra) {
                Ok(v) => return Some(v),
                Err(_) => {
                    if retry_with_lax_check {
                        new_extra.check = SerCheck::Lax;
                        if let Ok(v) = selector(&selected_serializer, &new_extra) {
                            return Some(v);
                        }
                    }
                }
            }
        }
    }

    None
}

impl TypeSerializer for UnionSerializer {
    fn to_python(
        &self,
        value: &Bound<'_, PyAny>,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        union_serialize(
            |comb_serializer, new_extra| comb_serializer.to_python(value, include, exclude, new_extra),
            |v| v.map_or_else(|| infer_to_python(value, include, exclude, extra), Ok),
            extra,
            &self.choices,
            self.retry_with_lax_check(),
        )?
    }

    fn json_key<'a>(&self, key: &'a Bound<'_, PyAny>, extra: &Extra) -> PyResult<Cow<'a, str>> {
        union_serialize(
            |comb_serializer, new_extra| comb_serializer.json_key(key, new_extra),
            |v| v.map_or_else(|| infer_json_key(key, extra), Ok),
            extra,
            &self.choices,
            self.retry_with_lax_check(),
        )?
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Bound<'_, PyAny>,
        serializer: S,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        union_serialize(
            |comb_serializer, new_extra| comb_serializer.to_python(value, include, exclude, new_extra),
            |v| {
                infer_serialize(
                    v.as_ref().map_or(value, |v| v.bind(value.py())),
                    serializer,
                    None,
                    None,
                    extra,
                )
            },
            extra,
            &self.choices,
            self.retry_with_lax_check(),
        )
        .map_err(|err| serde::ser::Error::custom(err.to_string()))?
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn retry_with_lax_check(&self) -> bool {
        self.choices.iter().any(CombinedSerializer::retry_with_lax_check)
    }
}

#[derive(Debug)]
pub struct TaggedUnionSerializer {
    discriminator: Discriminator,
    lookup: HashMap<String, usize>,
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

        // TODO: guarantee at least 1 choice
        let choices_map: Bound<PyDict> = schema.get_as_req(intern!(py, "choices"))?;
        let mut lookup = HashMap::with_capacity(choices_map.len());
        let mut choices = Vec::with_capacity(choices_map.len());

        for (idx, (choice_key, choice_schema)) in choices_map.into_iter().enumerate() {
            let serializer = CombinedSerializer::build(choice_schema.downcast()?, config, definitions)?;
            choices.push(serializer);
            lookup.insert(choice_key.to_string(), idx);
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

impl_py_gc_traverse!(TaggedUnionSerializer { discriminator, choices });

impl TypeSerializer for TaggedUnionSerializer {
    fn to_python(
        &self,
        value: &Bound<'_, PyAny>,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let to_python_selector = |comb_serializer: &CombinedSerializer, new_extra: &Extra| {
            comb_serializer.to_python(value, include, exclude, new_extra)
        };

        tagged_union_serialize(
            self.get_discriminator_value(value, extra),
            &self.lookup,
            to_python_selector,
            extra,
            &self.choices,
            self.retry_with_lax_check(),
        )
        .map_or_else(
            || {
                union_serialize(
                    to_python_selector,
                    |v| v.map_or_else(|| infer_to_python(value, include, exclude, extra), Ok),
                    extra,
                    &self.choices,
                    self.retry_with_lax_check(),
                )?
            },
            Ok,
        )
    }

    fn json_key<'a>(&self, key: &'a Bound<'_, PyAny>, extra: &Extra) -> PyResult<Cow<'a, str>> {
        let json_key_selector =
            |comb_serializer: &CombinedSerializer, new_extra: &Extra| comb_serializer.json_key(key, new_extra);

        tagged_union_serialize(
            self.get_discriminator_value(key, extra),
            &self.lookup,
            json_key_selector,
            extra,
            &self.choices,
            self.retry_with_lax_check(),
        )
        .map_or_else(
            || {
                union_serialize(
                    json_key_selector,
                    |v| v.map_or_else(|| infer_json_key(key, extra), Ok),
                    extra,
                    &self.choices,
                    self.retry_with_lax_check(),
                )?
            },
            Ok,
        )
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Bound<'_, PyAny>,
        serializer: S,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        let serde_selector = |comb_serializer: &CombinedSerializer, new_extra: &Extra| {
            comb_serializer.to_python(value, include, exclude, new_extra)
        };

        tagged_union_serialize(
            None,
            &self.lookup,
            serde_selector,
            extra,
            &self.choices,
            self.retry_with_lax_check(),
        )
        .map_or_else(
            || {
                union_serialize(
                    serde_selector,
                    |v| {
                        infer_serialize(
                            v.as_ref().map_or(value, |v| v.bind(value.py())),
                            serializer,
                            None,
                            None,
                            extra,
                        )
                    },
                    extra,
                    &self.choices,
                    self.retry_with_lax_check(),
                )
                .map_err(|err| serde::ser::Error::custom(err.to_string()))?
            },
            |v| infer_serialize(v.bind(value.py()), serializer, None, None, extra),
        )
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn retry_with_lax_check(&self) -> bool {
        self.choices.iter().any(CombinedSerializer::retry_with_lax_check)
    }
}

impl TaggedUnionSerializer {
    fn get_discriminator_value(&self, value: &Bound<'_, PyAny>, extra: &Extra) -> Option<Py<PyAny>> {
        let py = value.py();
        let discriminator_value = match &self.discriminator {
            Discriminator::LookupKey(lookup_key) => {
                // we're pretty lax here, we allow either dict[key] or object.key, as we very well could
                // be doing a discriminator lookup on a typed dict, and there's no good way to check that
                // at this point. we could be more strict and only do this in lax mode...
                let getattr_result = match value.is_instance_of::<PyDict>() {
                    true => {
                        let value_dict = value.downcast::<PyDict>().unwrap();
                        lookup_key.py_get_dict_item(value_dict).ok()
                    }
                    false => lookup_key.simple_py_get_attr(value).ok(),
                };
                getattr_result.and_then(|opt| opt.map(|(_, bound)| bound.to_object(py)))
            }
            Discriminator::Function(func) => func.call1(py, (value,)).ok(),
        };
        if discriminator_value.is_none() {
            let value_str = truncate_safe_repr(value, None);

            // If extra.check is SerCheck::None, we're in a top-level union. We should thus raise this warning
            if extra.check == SerCheck::None {
                extra.warnings.custom_warning(
                    format!(
                        "Failed to get discriminator value for tagged union serialization with value `{value_str}` - defaulting to left to right union serialization."
                    )
                );
            }
        }
        discriminator_value
    }
}
