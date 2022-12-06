use std::borrow::Cow;

use pyo3::prelude::*;
use pyo3::types::{PyDate, PyDateTime, PyDict, PyTime};

use crate::input::{pydate_as_date, pydatetime_as_datetime, pytime_as_time};

use super::any::{fallback_serialize, fallback_to_python_json, json_key};
use super::shared::{py_err_se_err, BuildSerializer, CombinedSerializer, Extra, SerMode, TypeSerializer};

pub(crate) fn datetime_to_string(py_dt: &PyDateTime) -> PyResult<String> {
    let dt = pydatetime_as_datetime(py_dt)?;
    Ok(dt.to_string())
}

pub(crate) fn date_to_string(py_date: &PyDate) -> PyResult<String> {
    let date = pydate_as_date!(py_date);
    Ok(date.to_string())
}

pub(crate) fn time_to_string(py_time: &PyTime) -> PyResult<String> {
    let time = pytime_as_time!(py_time);
    Ok(time.to_string())
}

macro_rules! build_serializer {
    ($struct_name:ident, $expected_type:literal, $cast_as:ty, $convert_func:ident) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name;

        impl BuildSerializer for $struct_name {
            const EXPECTED_TYPE: &'static str = $expected_type;

            fn build(_schema: &PyDict, _config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
                Ok(Self {}.into())
            }
        }

        impl TypeSerializer for $struct_name {
            fn to_python(
                &self,
                value: &PyAny,
                _include: Option<&PyAny>,
                _exclude: Option<&PyAny>,
                extra: &Extra,
            ) -> PyResult<PyObject> {
                let py = value.py();
                match extra.mode {
                    SerMode::Json => match value.cast_as::<$cast_as>() {
                        Ok(py_value) => {
                            let s = $convert_func(py_value)?;
                            Ok(s.into_py(py))
                        }
                        Err(_) => {
                            extra.warnings.fallback_slow(Self::EXPECTED_TYPE, value);
                            fallback_to_python_json(value, extra)
                        }
                    },
                    _ => Ok(value.into_py(py)),
                }
            }

            fn json_key<'py>(&self, key: &'py PyAny, extra: &Extra) -> PyResult<Cow<'py, str>> {
                match key.cast_as::<$cast_as>() {
                    Ok(py_value) => Ok(Cow::Owned($convert_func(py_value)?)),
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
                match value.cast_as::<$cast_as>() {
                    Ok(py_value) => {
                        let s = $convert_func(py_value).map_err(py_err_se_err)?;
                        serializer.serialize_str(&s)
                    }
                    Err(_) => {
                        extra.warnings.fallback_slow(Self::EXPECTED_TYPE, value);
                        fallback_serialize(value, serializer, extra)
                    }
                }
            }
        }
    };
}

build_serializer!(DatetimeSerializer, "datetime", PyDateTime, datetime_to_string);
build_serializer!(DateSerializer, "date", PyDate, date_to_string);
build_serializer!(TimeSerializer, "time", PyTime, time_to_string);
