use std::borrow::Cow;

use pyo3::prelude::*;
use pyo3::types::{PyDate, PyDateTime, PyDict, PyTime};

use super::{
    infer_json_key, infer_serialize, infer_to_python, BuildSerializer, CombinedSerializer, Extra, SerMode,
    TypeSerializer,
};
use crate::definitions::DefinitionsBuilder;
use crate::input::{pydate_as_date, pydatetime_as_datetime, pytime_as_time};
use crate::serializers::config::{FromConfig, TemporalMode};
use crate::PydanticSerializationUnexpectedValue;

pub(crate) fn datetime_to_string(py_dt: &Bound<'_, PyDateTime>) -> PyResult<String> {
    pydatetime_as_datetime(py_dt).map(|dt| dt.to_string())
}

pub(crate) fn datetime_to_seconds(py_dt: &Bound<'_, PyDateTime>) -> PyResult<i64> {
    pydatetime_as_datetime(py_dt).map(|dt| dt.timestamp())
}

pub(crate) fn datetime_to_milliseconds(py_dt: &Bound<'_, PyDateTime>) -> PyResult<i64> {
    pydatetime_as_datetime(py_dt).map(|dt| dt.timestamp_ms())
}

pub(crate) fn date_to_seconds(py_date: &Bound<'_, PyDate>) -> PyResult<i64> {
    pydate_as_date(py_date).map(|dt| dt.timestamp())
}
pub(crate) fn date_to_milliseconds(py_date: &Bound<'_, PyDate>) -> PyResult<i64> {
    pydate_as_date(py_date).map(|dt| dt.timestamp_ms())
}

pub(crate) fn date_to_string(py_date: &Bound<'_, PyDate>) -> PyResult<String> {
    pydate_as_date(py_date).map(|dt| dt.to_string())
}

pub(crate) fn time_to_string(py_time: &Bound<'_, PyTime>) -> PyResult<String> {
    pytime_as_time(py_time, None).map(|dt| dt.to_string())
}

pub(crate) fn time_to_seconds(py_time: &Bound<'_, PyTime>) -> PyResult<u32> {
    pytime_as_time(py_time, None).map(|t| t.total_seconds())
}

pub(crate) fn time_to_milliseconds(py_time: &Bound<'_, PyTime>) -> PyResult<u32> {
    pytime_as_time(py_time, None).map(|t| t.total_ms())
}

fn downcast_date_reject_datetime<'a, 'py>(py_date: &'a Bound<'py, PyAny>) -> PyResult<&'a Bound<'py, PyDate>> {
    if let Ok(py_date) = py_date.downcast::<PyDate>() {
        // because `datetime` is a subclass of `date` we have to check that the value is not a
        // `datetime` to avoid lossy serialization
        if !py_date.is_instance_of::<PyDateTime>() {
            return Ok(py_date);
        }
    }

    Err(PydanticSerializationUnexpectedValue::new_from_msg(None).to_py_err())
}

#[derive(Debug)]
pub struct DatetimeSerializer {
    temporal_mode: TemporalMode,
}

impl BuildSerializer for DatetimeSerializer {
    const EXPECTED_TYPE: &'static str = "datetime";

    fn build(
        _schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let temporal_mode = TemporalMode::from_config(config)?;
        Ok(Self { temporal_mode }.into())
    }
}
impl_py_gc_traverse!(DatetimeSerializer {});

impl TypeSerializer for DatetimeSerializer {
    fn to_python(
        &self,
        value: &Bound<'_, PyAny>,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        match extra.mode {
            SerMode::Json => match PyAnyMethods::downcast::<PyDateTime>(value) {
                Ok(py_value) => Ok(self.temporal_mode.datetime_to_json(value.py(), py_value)?),
                Err(_) => {
                    extra.warnings.on_fallback_py(self.get_name(), value, extra)?;
                    infer_to_python(value, include, exclude, extra)
                }
            },
            _ => infer_to_python(value, include, exclude, extra),
        }
    }

    fn json_key<'a>(&self, key: &'a Bound<'_, PyAny>, extra: &Extra) -> PyResult<Cow<'a, str>> {
        match PyAnyMethods::downcast::<PyDateTime>(key) {
            Ok(py_value) => Ok(self.temporal_mode.datetime_json_key(py_value)?),
            Err(_) => {
                extra.warnings.on_fallback_py(self.get_name(), key, extra)?;
                infer_json_key(key, extra)
            }
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Bound<'_, PyAny>,
        serializer: S,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        match PyAnyMethods::downcast::<PyDateTime>(value) {
            Ok(py_value) => self.temporal_mode.datetime_serialize(py_value, serializer),
            Err(_) => {
                extra.warnings.on_fallback_ser::<S>(self.get_name(), value, extra)?;
                infer_serialize(value, serializer, include, exclude, extra)
            }
        }
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}

#[derive(Debug)]
pub struct DateSerializer {
    temporal_mode: TemporalMode,
}

impl BuildSerializer for DateSerializer {
    const EXPECTED_TYPE: &'static str = "date";

    fn build(
        _schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let temporal_mode = TemporalMode::from_config(config)?;
        Ok(Self { temporal_mode }.into())
    }
}
impl_py_gc_traverse!(DateSerializer {});

impl TypeSerializer for DateSerializer {
    fn to_python(
        &self,
        value: &Bound<'_, PyAny>,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        match extra.mode {
            SerMode::Json => match downcast_date_reject_datetime(value) {
                Ok(py_value) => Ok(self.temporal_mode.date_to_json(value.py(), py_value)?),
                Err(_) => {
                    extra.warnings.on_fallback_py(self.get_name(), value, extra)?;
                    infer_to_python(value, include, exclude, extra)
                }
            },
            _ => infer_to_python(value, include, exclude, extra),
        }
    }

    fn json_key<'a>(&self, key: &'a Bound<'_, PyAny>, extra: &Extra) -> PyResult<Cow<'a, str>> {
        match downcast_date_reject_datetime(key) {
            Ok(py_value) => Ok(self.temporal_mode.date_json_key(py_value)?),
            Err(_) => {
                extra.warnings.on_fallback_py(self.get_name(), key, extra)?;
                infer_json_key(key, extra)
            }
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Bound<'_, PyAny>,
        serializer: S,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        match downcast_date_reject_datetime(value) {
            Ok(py_value) => self.temporal_mode.date_serialize(py_value, serializer),
            Err(_) => {
                extra.warnings.on_fallback_ser::<S>(self.get_name(), value, extra)?;
                infer_serialize(value, serializer, include, exclude, extra)
            }
        }
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}

#[derive(Debug)]
pub struct TimeSerializer {
    temporal_mode: TemporalMode,
}

impl BuildSerializer for TimeSerializer {
    const EXPECTED_TYPE: &'static str = "time";

    fn build(
        _schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let temporal_mode = TemporalMode::from_config(config)?;
        Ok(Self { temporal_mode }.into())
    }
}
impl_py_gc_traverse!(TimeSerializer {});

impl TypeSerializer for TimeSerializer {
    fn to_python(
        &self,
        value: &Bound<'_, PyAny>,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        match extra.mode {
            SerMode::Json => match PyAnyMethods::downcast::<PyTime>(value) {
                Ok(py_value) => Ok(self.temporal_mode.time_to_json(value.py(), py_value)?),
                Err(_) => {
                    extra.warnings.on_fallback_py(self.get_name(), value, extra)?;
                    infer_to_python(value, include, exclude, extra)
                }
            },
            _ => infer_to_python(value, include, exclude, extra),
        }
    }

    fn json_key<'a>(&self, key: &'a Bound<'_, PyAny>, extra: &Extra) -> PyResult<Cow<'a, str>> {
        match PyAnyMethods::downcast::<PyTime>(key) {
            Ok(py_value) => Ok(self.temporal_mode.time_json_key(py_value)?),
            Err(_) => {
                extra.warnings.on_fallback_py(self.get_name(), key, extra)?;
                infer_json_key(key, extra)
            }
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Bound<'_, PyAny>,
        serializer: S,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        match PyAnyMethods::downcast::<PyTime>(value) {
            Ok(py_value) => self.temporal_mode.time_serialize(py_value, serializer),
            Err(_) => {
                extra.warnings.on_fallback_ser::<S>(self.get_name(), value, extra)?;
                infer_serialize(value, serializer, include, exclude, extra)
            }
        }
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}
