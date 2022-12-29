use std::borrow::Cow;

use pyo3::prelude::*;
use pyo3::types::{PyDelta, PyDict};
use pyo3::{intern, PyNativeType};

use crate::build_context::BuildContext;
use crate::build_tools::{py_err, SchemaDict};
use crate::input::pytimedelta_as_duration;

use super::any::{fallback_serialize, fallback_to_python_json, json_key};
use super::shared::{py_err_se_err, BuildSerializer, CombinedSerializer, Extra, SerMode, TypeSerializer};

#[derive(Debug, Clone, Copy)]
pub enum TimedeltaMode {
    Iso8601,
    Float,
}

impl TimedeltaMode {
    pub fn from_config(config: Option<&PyDict>) -> PyResult<Self> {
        let raw_mode: Option<&str> = match config {
            Some(c) => c.get_as::<&str>(intern!(c.py(), "serialization_timedelta_mode"))?,
            None => None,
        };
        match raw_mode {
            Some("iso8601") => Ok(Self::Iso8601),
            Some("float") => Ok(Self::Float),
            Some(s) => py_err!(
                "Invalid timedelta serialization mode: `{}`, expected `iso8601` or `float`",
                s
            ),
            None => Ok(Self::Iso8601),
        }
    }

    fn total_seconds(py_timedelta: &PyDelta) -> PyResult<&PyAny> {
        py_timedelta.call_method0(intern!(py_timedelta.py(), "total_seconds"))
    }

    pub fn timedelta_to_json(&self, py_timedelta: &PyDelta) -> PyResult<PyObject> {
        let py = py_timedelta.py();
        match self {
            Self::Iso8601 => {
                let d = pytimedelta_as_duration(py_timedelta);
                Ok(d.to_string().into_py(py))
            }
            Self::Float => {
                let seconds = Self::total_seconds(py_timedelta)?;
                Ok(seconds.into_py(py))
            }
        }
    }

    pub fn json_key<'py>(&self, py_timedelta: &PyDelta) -> PyResult<Cow<'py, str>> {
        match self {
            Self::Iso8601 => {
                let d = pytimedelta_as_duration(py_timedelta);
                Ok(d.to_string().into())
            }
            Self::Float => {
                let seconds: f64 = Self::total_seconds(py_timedelta)?.extract()?;
                Ok(seconds.to_string().into())
            }
        }
    }

    pub fn timedelta_serialize<S: serde::ser::Serializer>(
        &self,
        py_timedelta: &PyDelta,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match self {
            Self::Iso8601 => {
                let d = pytimedelta_as_duration(py_timedelta);
                serializer.serialize_str(&d.to_string())
            }
            Self::Float => {
                let seconds = Self::total_seconds(py_timedelta).map_err(py_err_se_err)?;
                let seconds: f64 = seconds.extract().map_err(py_err_se_err)?;
                serializer.serialize_f64(seconds)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimeDeltaSerializer;

impl BuildSerializer for TimeDeltaSerializer {
    const EXPECTED_TYPE: &'static str = "timedelta";

    fn build(
        _schema: &PyDict,
        _config: Option<&PyDict>,
        _build_context: &mut BuildContext<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        Ok(Self {}.into())
    }
}

impl TypeSerializer for TimeDeltaSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let py = value.py();
        match extra.mode {
            SerMode::Json => match value.cast_as::<PyDelta>() {
                Ok(py_timedelta) => extra.timedelta_mode.timedelta_to_json(py_timedelta),
                Err(_) => {
                    extra.warnings.fallback_slow(Self::EXPECTED_TYPE, value);
                    fallback_to_python_json(value, extra)
                }
            },
            _ => Ok(value.into_py(py)),
        }
    }

    fn json_key<'py>(&self, key: &'py PyAny, extra: &Extra) -> PyResult<Cow<'py, str>> {
        match key.cast_as::<PyDelta>() {
            Ok(py_timedelta) => extra.timedelta_mode.json_key(py_timedelta),
            Err(_) => {
                extra.warnings.fallback_slow(Self::EXPECTED_TYPE, key);
                json_key(key, extra)
            }
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        _include: Option<&PyAny>,
        _exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        match value.cast_as::<PyDelta>() {
            Ok(py_timedelta) => extra.timedelta_mode.timedelta_serialize(py_timedelta, serializer),
            Err(_) => {
                extra.warnings.fallback_slow(Self::EXPECTED_TYPE, value);
                fallback_serialize(value, serializer, extra)
            }
        }
    }
}
