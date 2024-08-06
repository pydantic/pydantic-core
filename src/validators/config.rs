use std::borrow::Cow;
use std::str::FromStr;

use base64::Engine;
use pyo3::types::{PyDict, PyString};
use pyo3::{intern, prelude::*};

use crate::errors::ErrorType;
use crate::input::EitherBytes;
use crate::serializers::BytesMode;
use crate::tools::SchemaDict;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValBytesMode {
    pub ser: BytesMode,
}

impl ValBytesMode {
    pub fn from_config(config: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let Some(config_dict) = config else {
            return Ok(Self::default());
        };
        let raw_mode = config_dict.get_as::<Bound<'_, PyString>>(intern!(config_dict.py(), "val_json_bytes"))?;
        let ser_mode = raw_mode.map_or_else(|| Ok(BytesMode::default()), |raw| BytesMode::from_str(&raw.to_cow()?))?;
        Ok(Self { ser: ser_mode })
    }

    pub fn deserialize_string<'py>(self, s: &str) -> Result<EitherBytes<'_, 'py>, ErrorType> {
        match self.ser {
            BytesMode::Utf8 => Ok(EitherBytes::Cow(Cow::Borrowed(s.as_bytes()))),
            BytesMode::Base64 => {
                fn decode(input: &str) -> Result<Vec<u8>, ErrorType> {
                    base64::engine::general_purpose::URL_SAFE.decode(input).map_err(|err| {
                        ErrorType::BytesInvalidEncoding {
                            encoding: "base64".to_string(),
                            encoding_error: err.to_string(),
                            context: None,
                        }
                    })
                }
                let result = if s.contains(|c| c == '+' || c == '/') {
                    let replaced: String = s
                        .chars()
                        .map(|c| match c {
                            '+' => '-',
                            '/' => '_',
                            _ => c,
                        })
                        .collect();
                    decode(&replaced)
                } else {
                    decode(s)
                };
                result.map(EitherBytes::from)
            }
            BytesMode::Hex => match hex::decode(s) {
                Ok(vec) => Ok(EitherBytes::from(vec)),
                Err(err) => Err(ErrorType::BytesInvalidEncoding {
                    encoding: "hex".to_string(),
                    encoding_error: err.to_string(),
                    context: None,
                }),
            },
        }
    }
}
