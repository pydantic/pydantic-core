use std::fmt;

use indexmap::IndexMap;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PySet};
use serde::de::{Deserialize, DeserializeSeed, Error as SerdeError, MapAccess, SeqAccess, Visitor};

use crate::build_tools::py_err;

#[derive(Copy, Clone, Debug)]
pub enum JsonType {
    Null = 0b10000000,
    Bool = 0b01000000,
    Int = 0b00100000,
    Float = 0b00010000,
    String = 0b00001000,
    Array = 0b00000100,
    Object = 0b00000010,
}

impl JsonType {
    pub fn combine(set: &PySet) -> PyResult<u8> {
        set.iter().map(Self::try_from).try_fold(0u8, |a, b| Ok(a | b? as u8))
    }

    pub fn matches(&self, mask: u8) -> bool {
        *self as u8 & mask > 0
    }
}

impl TryFrom<&PyAny> for JsonType {
    type Error = PyErr;

    fn try_from(value: &PyAny) -> PyResult<Self> {
        let s: &str = value.extract()?;
        match s {
            "null" => Ok(Self::Null),
            "bool" => Ok(Self::Bool),
            "int" => Ok(Self::Int),
            "float" => Ok(Self::Float),
            "str" => Ok(Self::String),
            "list" => Ok(Self::Array),
            "dict" => Ok(Self::Object),
            _ => py_err!("Invalid json type: {}", s),
        }
    }
}

/// similar to serde `Value` but with int and float split
#[derive(Clone, Debug)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(JsonArray),
    Object(JsonObject),
}
pub type JsonArray = Vec<JsonValue>;
pub type JsonObject = IndexMap<String, JsonValue>;

impl ToPyObject for JsonValue {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            Self::Null => py.None(),
            Self::Bool(b) => b.into_py(py),
            Self::Int(i) => i.into_py(py),
            Self::Float(f) => f.into_py(py),
            Self::String(s) => s.into_py(py),
            Self::Array(v) => PyList::new(py, v.iter().map(|v| v.to_object(py))).into_py(py),
            Self::Object(o) => {
                let dict = PyDict::new(py);
                for (k, v) in o.iter() {
                    dict.set_item(k, v.to_object(py)).unwrap();
                }
                dict.into_py(py)
            }
        }
    }
}

impl<'de> Deserialize<'de> for JsonValue {
    fn deserialize<D>(deserializer: D) -> Result<JsonValue, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct JsonVisitor;

        impl<'de> Visitor<'de> for JsonVisitor {
            type Value = JsonValue;

            #[cfg_attr(has_no_coverage, no_coverage)]
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid JSON value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<JsonValue, E> {
                Ok(JsonValue::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<JsonValue, E> {
                Ok(JsonValue::Int(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<JsonValue, E> {
                Ok(JsonValue::Int(value as i64))
            }

            fn visit_f64<E>(self, value: f64) -> Result<JsonValue, E> {
                Ok(JsonValue::Float(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<JsonValue, E>
            where
                E: SerdeError,
            {
                Ok(JsonValue::String(value.to_string()))
            }

            #[cfg_attr(has_no_coverage, no_coverage)]
            fn visit_string<E>(self, _: String) -> Result<JsonValue, E> {
                unreachable!()
            }

            #[cfg_attr(has_no_coverage, no_coverage)]
            fn visit_none<E>(self) -> Result<JsonValue, E> {
                unreachable!()
            }

            #[cfg_attr(has_no_coverage, no_coverage)]
            fn visit_some<D>(self, _: D) -> Result<JsonValue, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                unreachable!()
            }

            fn visit_unit<E>(self) -> Result<JsonValue, E> {
                Ok(JsonValue::Null)
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<JsonValue, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }

                Ok(JsonValue::Array(vec))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<JsonValue, V::Error>
            where
                V: MapAccess<'de>,
            {
                match visitor.next_key_seed(KeyDeserializer)? {
                    Some(first_key) => {
                        let mut values = IndexMap::new();

                        values.insert(first_key, visitor.next_value()?);
                        while let Some((key, value)) = visitor.next_entry()? {
                            values.insert(key, value);
                        }
                        Ok(JsonValue::Object(values))
                    }
                    None => Ok(JsonValue::Object(IndexMap::new())),
                }
            }
        }

        deserializer.deserialize_any(JsonVisitor)
    }
}

struct KeyDeserializer;

impl<'de> DeserializeSeed<'de> for KeyDeserializer {
    type Value = String;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

impl<'de> Visitor<'de> for KeyDeserializer {
    type Value = String;

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string key")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(s.to_string())
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn visit_string<E>(self, _: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        unreachable!()
    }
}
