use std::borrow::Cow;
use std::str::FromStr;

use base64::Engine;
use pyo3::exceptions::PyValueError;
use pyo3::types::{PyDict, PyString};
use pyo3::{intern, prelude::*};

use crate::input::EitherBytes;
use crate::serializers::config::BytesMode;
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

    pub fn deserialize_string<'py>(self, s: &str) -> PyResult<EitherBytes<'_, 'py>> {
        match self.ser {
            BytesMode::Utf8 => Ok(EitherBytes::Cow(Cow::Borrowed(s.as_bytes()))),
            BytesMode::Base64 => match base64::engine::general_purpose::URL_SAFE.decode(s) {
                Ok(bytes) => Ok(EitherBytes::from(bytes)),
                Err(err) => Err(PyValueError::new_err(format!("Base64 decode error: {err}"))),
            },
            BytesMode::Hex => Err(PyValueError::new_err("Hex deserialization is not supported")),
        }
    }
}
