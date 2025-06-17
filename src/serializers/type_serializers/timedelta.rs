use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::borrow::Cow;

use crate::definitions::DefinitionsBuilder;
use crate::input::EitherTimedelta;
use crate::serializers::config::{FromConfig, TemporalMode, TimedeltaMode};

use super::{
    infer_json_key, infer_serialize, infer_to_python, BuildSerializer, CombinedSerializer, Extra, SerMode,
    TypeSerializer,
};

#[derive(Debug)]
pub struct TimeDeltaSerializer {
    timedelta_mode: TimedeltaMode,
    temporal_mode: TemporalMode,
    prefer_timedelta_mode: bool,
}

pub enum EffectiveDeltaMode {
    Timedelta(TimedeltaMode),
    Temporal(TemporalMode),
}

impl BuildSerializer for TimeDeltaSerializer {
    const EXPECTED_TYPE: &'static str = "timedelta";

    fn build(
        _schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let timedelta_mode = TimedeltaMode::from_config(config)?;
        let temporal_mode = TemporalMode::from_config(config)?;

        let prefer_timedelta_mode = config
            .and_then(|cfg| cfg.contains(intern!(cfg.py(), "ser_json_timedelta")).ok())
            .unwrap_or(false);

        Ok(Self {
            timedelta_mode,
            temporal_mode,
            prefer_timedelta_mode,
        }
        .into())
    }
}

impl TimeDeltaSerializer {
    pub fn effective_delta_mode(&self) -> EffectiveDeltaMode {
        if self.prefer_timedelta_mode {
            EffectiveDeltaMode::Timedelta(self.timedelta_mode.clone())
        } else {
            EffectiveDeltaMode::Temporal(self.temporal_mode.clone())
        }
    }
}

impl_py_gc_traverse!(TimeDeltaSerializer {});

impl TypeSerializer for TimeDeltaSerializer {
    fn to_python(
        &self,
        value: &Bound<'_, PyAny>,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        match extra.mode {
            SerMode::Json => match EitherTimedelta::try_from(value) {
                Ok(either_timedelta) => match self.effective_delta_mode() {
                    EffectiveDeltaMode::Timedelta(timedelta_mode) => {
                        Ok(timedelta_mode.either_delta_to_json(value.py(), either_timedelta)?)
                    }
                    EffectiveDeltaMode::Temporal(temporal_mode) => {
                        Ok(temporal_mode.timedelta_to_json(value.py(), either_timedelta)?)
                    }
                },
                Err(_) => {
                    extra.warnings.on_fallback_py(self.get_name(), value, extra)?;
                    infer_to_python(value, include, exclude, extra)
                }
            },
            _ => infer_to_python(value, include, exclude, extra),
        }
    }

    fn json_key<'a>(&self, key: &'a Bound<'_, PyAny>, extra: &Extra) -> PyResult<Cow<'a, str>> {
        match EitherTimedelta::try_from(key) {
            Ok(either_timedelta) => match self.effective_delta_mode() {
                EffectiveDeltaMode::Timedelta(timedelta_mode) => timedelta_mode.json_key(key.py(), either_timedelta),
                EffectiveDeltaMode::Temporal(temporal_mode) => temporal_mode.timedelta_json_key(&either_timedelta),
            },
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
        match EitherTimedelta::try_from(value) {
            Ok(either_timedelta) => match self.effective_delta_mode() {
                EffectiveDeltaMode::Timedelta(timedelta_mode) => {
                    timedelta_mode.timedelta_serialize(value.py(), either_timedelta, serializer)
                }
                EffectiveDeltaMode::Temporal(temporal_mode) => {
                    temporal_mode.timedelta_serialize(either_timedelta, serializer)
                }
            },
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
