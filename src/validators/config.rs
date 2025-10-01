use std::borrow::Cow;
use std::convert::Infallible;
use std::str::FromStr;

use crate::build_tools::py_schema_err;
use crate::config::CoreConfig;
use crate::errors::ErrorType;
use crate::input::EitherBytes;
use crate::serializers::BytesMode;
use base64::engine::general_purpose::GeneralPurpose;
use base64::engine::{DecodePaddingMode, GeneralPurposeConfig};
use base64::{alphabet, DecodeError, Engine};
use pyo3::types::PyString;
use pyo3::{intern, prelude::*};
use speedate::TimestampUnit;

const URL_SAFE_OPTIONAL_PADDING: GeneralPurpose = GeneralPurpose::new(
    &alphabet::URL_SAFE,
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
);
const STANDARD_OPTIONAL_PADDING: GeneralPurpose = GeneralPurpose::new(
    &alphabet::STANDARD,
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalUnitMode {
    Seconds,
    Milliseconds,
    #[default]
    Infer,
}

impl FromStr for TemporalUnitMode {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "seconds" => Ok(Self::Seconds),
            "milliseconds" => Ok(Self::Milliseconds),
            "infer" => Ok(Self::Infer),

            s => py_schema_err!(
                "Invalid temporal_unit_mode serialization mode: `{}`, expected seconds, milliseconds or infer",
                s
            ),
        }
    }
}

impl FromPyObject<'_> for TemporalUnitMode {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        let s: &str = ob.extract()?;
        Self::from_str(s)
    }
}

impl<'py> IntoPyObject<'py> for TemporalUnitMode {
    type Target = PyString;

    type Output = Borrowed<'py, 'py, PyString>;

    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let s = match self {
            Self::Seconds => intern!(py, "seconds"),
            Self::Milliseconds => intern!(py, "milliseconds"),
            Self::Infer => intern!(py, "infer"),
        };
        Ok(s.as_borrowed())
    }
}

impl TemporalUnitMode {
    pub fn from_config(config: &CoreConfig) -> Self {
        config.val_temporal_unit.unwrap_or_default()
    }
}

impl From<TemporalUnitMode> for TimestampUnit {
    fn from(value: TemporalUnitMode) -> Self {
        match value {
            TemporalUnitMode::Seconds => TimestampUnit::Second,
            TemporalUnitMode::Milliseconds => TimestampUnit::Millisecond,
            TemporalUnitMode::Infer => TimestampUnit::Infer,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValBytesMode {
    pub ser: BytesMode,
}

impl FromPyObject<'_> for ValBytesMode {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        let s: &str = ob.extract()?;
        let ser = BytesMode::from_str(s)?;
        Ok(Self { ser })
    }
}

impl<'py> IntoPyObject<'py> for ValBytesMode {
    type Target = PyString;

    type Output = Borrowed<'py, 'py, PyString>;

    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let s = match self.ser {
            BytesMode::Utf8 => intern!(py, "utf-8"),
            BytesMode::Base64 => intern!(py, "base64"),
            BytesMode::Hex => intern!(py, "hex"),
        };
        Ok(s.as_borrowed())
    }
}

impl ValBytesMode {
    pub fn from_config(config: &CoreConfig) -> Self {
        config.val_json_bytes.unwrap_or_default()
        // let Some(config_dict) = config else {
        //     return Ok(Self::default());
        // };
        // let raw_mode = config_dict.get_as::<Bound<'_, PyString>>(intern!(config_dict.py(), "val_json_bytes"))?;
        // let ser_mode = raw_mode.map_or_else(|| Ok(BytesMode::default()), |raw| BytesMode::from_str(&raw.to_cow()?))?;
        // Ok(Self { ser: ser_mode })
    }

    pub fn deserialize_string<'py>(self, s: &str) -> Result<EitherBytes<'_, 'py>, ErrorType> {
        match self.ser {
            BytesMode::Utf8 => Ok(EitherBytes::Cow(Cow::Borrowed(s.as_bytes()))),
            BytesMode::Base64 => URL_SAFE_OPTIONAL_PADDING
                .decode(s)
                .or_else(|err| match err {
                    DecodeError::InvalidByte(_, b'/' | b'+') => STANDARD_OPTIONAL_PADDING.decode(s),
                    _ => Err(err),
                })
                .map(EitherBytes::from)
                .map_err(|err| ErrorType::BytesInvalidEncoding {
                    encoding: "base64".to_string(),
                    encoding_error: err.to_string(),
                    context: None,
                }),
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
