use std::fmt;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use indexmap::IndexMap;
use serde::de::{
    self, Deserialize, DeserializeSeed, EnumAccess, Expected, IntoDeserializer, MapAccess,
    SeqAccess, Unexpected, VariantAccess, Visitor,
};

use crate::errors::LocItem;
use super::traits::{ToLocItem, ListInput, DictInput, Input};

// taken from `serde_json`
// We only use our own error type; no need for From conversions provided by the
// standard library's try! macro. This reduces lines of LLVM IR by 4%.
macro_rules! tri {
    ($e:expr $(,)?) => {
        match $e {
            Ok(val) => val,
            Err(err) => return Err(err),
        }
    };
}

/// similar to serde `Value` but with int and float split
#[derive(Clone, Debug)]
pub enum JsonInput {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(JsonArray),
    Object(JsonObject),
}

#[derive(Clone, Debug)]
pub struct JsonArray(Vec<JsonInput>);

impl ToPyObject for JsonArray {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        self.0.iter().map(|v| v.to_object(py)).collect::<Vec<_>>().into_py(py)
    }
}

impl<'data> ListInput<'data> for &'data JsonArray {
    fn input_iter(&self) -> Box<dyn Iterator<Item = &'data dyn Input> + 'data> {
        Box::new(self.0.iter().map(|item| item as &dyn Input))
    }

    fn input_len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Clone, Debug)]
pub struct JsonObject(IndexMap<String, JsonInput>);

impl ToPyObject for JsonObject {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        for (k, v) in self.0.iter() {
            dict.set_item(k, v.to_object(py)).unwrap();
        }
        dict.into_py(py)
    }
}

impl<'data> DictInput<'data> for &'data JsonObject {
    fn input_iter(&self) -> Box<dyn Iterator<Item = (&'data dyn Input, &'data dyn Input)> + 'data> {
        Box::new(self.0.iter().map(|(k, v)| (k as &dyn Input, v as &dyn Input)))
    }

    fn input_get(&self, key: &str) -> Option<&'data dyn Input> {
        self.0.get(key).map(|item| item as &dyn Input)
    }

    fn input_len(&self) -> usize {
        self.0.len()
    }
}

impl ToPyObject for JsonInput {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            JsonInput::Null => py.None(),
            JsonInput::Bool(b) => b.into_py(py),
            JsonInput::Int(i) => i.into_py(py),
            JsonInput::Float(f) => f.into_py(py),
            JsonInput::String(s) => s.into_py(py),
            JsonInput::Array(v) => v.to_object(py),
            JsonInput::Object(o) => o.to_object(py),
        }
    }
}

impl ToLocItem for JsonInput {
    fn to_loc(&self) -> LocItem {
        match self {
            JsonInput::Int(i) => LocItem::I(i as usize),
            JsonInput::String(s) => LocItem::S(s.to_string()),
            v => LocItem::S(format!("{:?}", v)),
        }
    }
}

impl<'de> Deserialize<'de> for JsonInput {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<JsonInput, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct JsonInputVisitor;

        impl<'de> Visitor<'de> for JsonInputVisitor {
            type Value = JsonInput;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid JSON value")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<JsonInput, E> {
                Ok(JsonInput::Bool(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<JsonInput, E> {
                Ok(JsonInput::Int(value))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<JsonInput, E> {
                Ok(JsonInput::Int(value as i64))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<JsonInput, E> {
                Ok(JsonInput::Float(value))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<JsonInput, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(value.to_string())
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<JsonInput, E> {
                Ok(JsonInput::String(value))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<JsonInput, E> {
                Ok(JsonInput::Null)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<JsonInput, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<JsonInput, E> {
                Ok(JsonInput::Null)
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<JsonInput, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = tri!(visitor.next_element()) {
                    vec.push(elem);
                }

                Ok(JsonInput::Array(JsonArray(vec)))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<JsonInput, V::Error>
            where
                V: MapAccess<'de>,
            {
                unimplemented!()
                // match visitor.next_key_seed(KeyClassifier)? {
                //     Some(KeyClass::Map(first_key)) => {
                //         let mut values = IndexMap::new();
                //
                //         values.insert(first_key, tri!(visitor.next_value()));
                //         while let Some((key, value)) = tri!(visitor.next_entry()) {
                //             values.insert(key, value);
                //         }
                //
                //         Ok(JsonInput::Object(values))
                //     }
                //     None => Ok(JsonInput::Object(IndexMap::new())),
                // }
            }
        }

        deserializer.deserialize_any(JsonInputVisitor)
    }
}
