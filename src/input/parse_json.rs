use std::fmt;

use num_bigint::BigInt;
use serde::de::{Deserialize, DeserializeSeed, Error as SerdeError, MapAccess, SeqAccess, Visitor};

use crate::data_value::DataValue;
use crate::lazy_index_map::LazyIndexMap;

pub type JsonInput = DataValue;
pub type JsonArray = Vec<JsonInput>;
pub type JsonObject = LazyIndexMap<String, JsonInput>;

impl<'de> Deserialize<'de> for DataValue {
    fn deserialize<D>(deserializer: D) -> Result<DataValue, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct JsonVisitor;

        impl<'de> Visitor<'de> for JsonVisitor {
            type Value = DataValue;

            #[cfg_attr(has_no_coverage, no_coverage)]
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid JSON value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<DataValue, E> {
                Ok(DataValue::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<DataValue, E> {
                Ok(DataValue::Int(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<DataValue, E> {
                match i64::try_from(value) {
                    Ok(i) => Ok(DataValue::Int(i)),
                    Err(_) => Ok(DataValue::Uint(value)),
                }
            }

            fn visit_f64<E>(self, value: f64) -> Result<DataValue, E> {
                Ok(DataValue::Float(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<DataValue, E>
            where
                E: SerdeError,
            {
                Ok(DataValue::String(value.to_string()))
            }

            fn visit_string<E>(self, value: String) -> Result<DataValue, E> {
                Ok(DataValue::String(value))
            }

            #[cfg_attr(has_no_coverage, no_coverage)]
            fn visit_none<E>(self) -> Result<DataValue, E> {
                unreachable!()
            }

            #[cfg_attr(has_no_coverage, no_coverage)]
            fn visit_some<D>(self, _: D) -> Result<DataValue, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                unreachable!()
            }

            fn visit_unit<E>(self) -> Result<DataValue, E> {
                Ok(DataValue::Null)
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<DataValue, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }

                Ok(DataValue::Array(vec))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<DataValue, V::Error>
            where
                V: MapAccess<'de>,
            {
                const SERDE_JSON_NUMBER: &str = "$serde_json::private::Number";
                match visitor.next_key_seed(KeyDeserializer)? {
                    Some(first_key) => {
                        let mut values = LazyIndexMap::new();
                        let first_value = visitor.next_value()?;

                        // serde_json will parse arbitrary precision numbers into a map
                        // structure with a "number" key and a String value
                        'try_number: {
                            if first_key == SERDE_JSON_NUMBER {
                                // Just in case someone tries to actually store that key in a real map,
                                // keep parsing and continue as a map if so

                                if let Some((key, value)) = visitor.next_entry::<String, DataValue>()? {
                                    // Important to preserve order of the keys
                                    values.insert(first_key, first_value);
                                    values.insert(key, value);
                                    break 'try_number;
                                }

                                if let DataValue::String(s) = &first_value {
                                    // Normalize the string to either an int or float
                                    let normalized = if s.chars().any(|c| c == '.' || c == 'E' || c == 'e') {
                                        DataValue::Float(
                                            s.parse()
                                                .map_err(|e| V::Error::custom(format!("expected a float: {e}")))?,
                                        )
                                    } else if let Ok(i) = s.parse::<i64>() {
                                        DataValue::Int(i)
                                    } else if let Ok(big) = s.parse::<BigInt>() {
                                        DataValue::BigInt(big)
                                    } else {
                                        // Failed to normalize, just throw it in the map and continue
                                        values.insert(first_key, first_value);
                                        break 'try_number;
                                    };

                                    return Ok(normalized);
                                };
                            } else {
                                values.insert(first_key, first_value);
                            }
                        }

                        while let Some((key, value)) = visitor.next_entry()? {
                            values.insert(key, value);
                        }
                        Ok(DataValue::Object(Box::new(values)))
                    }
                    None => Ok(DataValue::Object(Box::new(LazyIndexMap::new()))),
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
