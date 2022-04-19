use std::fmt;
use std::marker::PhantomData;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::de::{DeserializeSeed, Deserializer, Error as SerdeError, MapAccess, SeqAccess, Visitor};
use serde_json::Deserializer as JsonDeserializer;

use super::traits::ToPy;
use crate::errors::{err_val_error, ErrorKind, ValResult};

pub fn parse_json(py: Python, json: &str) -> ValResult<PyObject> {
    let mut deserializer = JsonDeserializer::from_str(json);
    let seed = JsonValue::new(py);

    match seed.deserialize(&mut deserializer) {
        Ok(data) => Ok(data.to_object(py)),
        Err(e) => err_val_error!(
            py,
            json.to_string(),
            message = Some(e.to_string()),
            kind = ErrorKind::InvalidJson
        ),
    }
}

#[derive(Copy, Clone)]
struct JsonValue<'py> {
    py: Python<'py>,
}

impl<'py> JsonValue<'py> {
    fn new(py: Python<'py>) -> JsonValue<'py> {
        JsonValue { py }
    }
}

impl<'de, 'a> DeserializeSeed<'de> for JsonValue<'a> {
    type Value = PyObject;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de, 'py> Visitor<'de> for JsonValue<'py> {
    type Value = PyObject;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("any valid JSON value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        Ok(value.to_object(self.py))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        Ok(value.to_object(self.py))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        Ok(value.to_object(self.py))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        Ok(value.to_object(self.py))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        Ok(value.to_object(self.py))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(self.py.None())
    }

    fn visit_seq<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut elements = Vec::with_capacity(access.size_hint().unwrap_or(0));

        while let Some(elem) = access.next_element_seed(self)? {
            elements.push(elem);
        }

        Ok(elements.to_object(self.py))
    }

    fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let dict = PyDict::new(self.py);

        while let Some((key, value)) = access.next_entry_seed(PhantomData::<String>, self)? {
            dict.set_item(key, value).unwrap();
        }
        Ok(dict.to_object(self.py))
    }
}
