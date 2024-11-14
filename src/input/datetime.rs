use pyo3::intern;
use pyo3::prelude::*;

use pyo3::exceptions::PyValueError;
use pyo3::pyclass::CompareOp;
use pyo3::types::{PyDate, PyDateTime, PyDelta, PyDeltaAccess, PyDict, PyTime, PyTzInfo};
use speedate::MicrosecondsPrecisionOverflowBehavior;
use speedate::{Date, DateTime, Duration, ParseError, Time, TimeConfig};
use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use strum::EnumMessage;

use super::Input;
use crate::errors::ToErrorValue;
use crate::errors::{ErrorType, ValError, ValResult};
use crate::tools::py_err;

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum EitherDate<'a> {
    Raw(Date),
    Py(Bound<'a, PyDate>),
}

impl<'a> From<Date> for EitherDate<'a> {
    fn from(date: Date) -> Self {
        Self::Raw(date)
    }
}

impl<'a> From<Bound<'a, PyDate>> for EitherDate<'a> {
    fn from(date: Bound<'a, PyDate>) -> Self {
        Self::Py(date)
    }
}

pub fn pydate_as_date(py_date: &Bound<'_, PyAny>) -> PyResult<Date> {
    let py = py_date.py();
    Ok(Date {
        year: py_date.getattr(intern!(py, "year"))?.extract()?,
        month: py_date.getattr(intern!(py, "month"))?.extract()?,
        day: py_date.getattr(intern!(py, "day"))?.extract()?,
    })
}

impl<'a> EitherDate<'a> {
    pub fn as_raw(&self) -> PyResult<Date> {
        match self {
            Self::Raw(date) => Ok(date.clone()),
            Self::Py(py_date) => pydate_as_date(py_date),
        }
    }

    pub fn try_into_pyobject(self, py: Python<'_>) -> PyResult<PyObject> {
        let date = match self {
            Self::Py(date) => Ok(date),
            Self::Raw(date) => PyDate::new(py, date.year.into(), date.month, date.day),
        }?;
        Ok(date.into_pyobject(py))
    }
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum EitherTime<'a> {
    Raw(Time),
    Py(Bound<'a, PyTime>),
}

impl<'a> From<Time> for EitherTime<'a> {
    fn from(time: Time) -> Self {
        Self::Raw(time)
    }
}

impl<'a> From<Bound<'a, PyTime>> for EitherTime<'a> {
    fn from(time: Bound<'a, PyTime>) -> Self {
        Self::Py(time)
    }
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum EitherTimedelta<'a> {
    Raw(Duration),
    PyExact(Bound<'a, PyDelta>),
    PySubclass(Bound<'a, PyDelta>),
}

impl<'a> From<Duration> for EitherTimedelta<'a> {
    fn from(timedelta: Duration) -> Self {
        Self::Raw(timedelta)
    }
}

impl<'a> EitherTimedelta<'a> {
    pub fn to_duration(&self) -> PyResult<Duration> {
        match self {
            Self::Raw(timedelta) => Ok(timedelta.clone()),
            Self::PyExact(py_timedelta) => Ok(pytimedelta_exact_as_duration(py_timedelta)),
            Self::PySubclass(py_timedelta) => pytimedelta_subclass_as_duration(py_timedelta),
        }
    }

    pub fn try_into_pyobject(&self, py: Python<'a>) -> PyResult<Bound<'a, PyDelta>> {
        match self {
            Self::PyExact(timedelta) => Ok(timedelta.clone()),
            Self::PySubclass(timedelta) => Ok(timedelta.clone()),
            Self::Raw(duration) => duration_as_pytimedelta(py, duration),
        }
    }
}

impl<'a> TryFrom<&'_ Bound<'a, PyAny>> for EitherTimedelta<'a> {
    type Error = PyErr;

    fn try_from(value: &Bound<'a, PyAny>) -> PyResult<Self> {
        if let Ok(dt) = value.downcast_exact() {
            Ok(EitherTimedelta::PyExact(dt.clone()))
        } else {
            let dt = value.downcast()?;
            Ok(EitherTimedelta::PySubclass(dt.clone()))
        }
    }
}

pub fn pytimedelta_exact_as_duration(py_timedelta: &Bound<'_, PyDelta>) -> Duration {
    // see https://docs.python.org/3/c-api/datetime.html#c.PyDateTime_DELTA_GET_DAYS
    // days can be negative, but seconds and microseconds are always positive.
    let mut days = py_timedelta.get_days(); // -999999999 to 999999999
    let mut seconds = py_timedelta.get_seconds(); // 0 through 86399
    let mut microseconds = py_timedelta.get_microseconds(); // 0 through 999999
    let positive = days >= 0;
    if !positive {
        // negative timedelta, we need to adjust values to match duration logic
        if microseconds != 0 {
            seconds += 1;
            microseconds = (microseconds - 1_000_000).abs();
        }
        if seconds != 0 {
            days += 1;
            seconds = (seconds - 86400).abs();
        }
        days = days.abs();
    }
    // we can safely "unwrap" since the methods above guarantee values are in the correct ranges.
    Duration::new(positive, days as u32, seconds as u32, microseconds as u32).unwrap()
}

pub fn pytimedelta_subclass_as_duration(py_timedelta: &Bound<'_, PyDelta>) -> PyResult<Duration> {
    let total_seconds: f64 = py_timedelta
        .call_method0(intern!(py_timedelta.py(), "total_seconds"))?
        .extract()?;
    if total_seconds.is_nan() {
        return py_err!(PyValueError; "NaN values not permitted");
    }
    let positive = total_seconds >= 0_f64;
    let total_seconds = total_seconds.abs();
    let microsecond = total_seconds.fract() * 1_000_000.0;
    let days = (total_seconds / 86400f64) as u32;
    let seconds = total_seconds as u64 % 86400;
    Duration::new(positive, days, seconds as u32, microsecond.round() as u32)
        .map_err(|err| PyValueError::new_err(err.to_string()))
}

pub fn duration_as_pytimedelta<'py>(py: Python<'py>, duration: &Duration) -> PyResult<Bound<'py, PyDelta>> {
    let sign = if duration.positive { 1 } else { -1 };
    PyDelta::new(
        py,
        sign * duration.day as i32,
        sign * duration.second as i32,
        sign * duration.microsecond as i32,
        true,
    )
}

pub fn pytime_as_time(py_time: &Bound<'_, PyAny>, py_dt: Option<&Bound<'_, PyAny>>) -> PyResult<Time> {
    let py = py_time.py();

    let tzinfo = py_time.getattr(intern!(py, "tzinfo"))?;
    let tz_offset: Option<i32> = if PyAnyMethods::is_none(&tzinfo) {
        None
    } else {
        let offset_delta = tzinfo.call_method1(intern!(py, "utcoffset"), (py_dt,))?;
        // as per the docs, utcoffset() can return None
        if PyAnyMethods::is_none(&offset_delta) {
            None
        } else {
            let offset_seconds: f64 = offset_delta.call_method0(intern!(py, "total_seconds"))?.extract()?;
            Some(offset_seconds.round() as i32)
        }
    };

    Ok(Time {
        hour: py_time.getattr(intern!(py, "hour"))?.extract()?,
        minute: py_time.getattr(intern!(py, "minute"))?.extract()?,
        second: py_time.getattr(intern!(py, "second"))?.extract()?,
        microsecond: py_time.getattr(intern!(py, "microsecond"))?.extract()?,
        tz_offset,
    })
}

impl<'a> EitherTime<'a> {
    pub fn as_raw(&self) -> PyResult<Time> {
        match self {
            Self::Raw(time) => Ok(time.clone()),
            Self::Py(py_time) => pytime_as_time(py_time, None),
        }
    }

    pub fn try_into_pyobject(self, py: Python<'_>) -> PyResult<PyObject> {
        let time = match self {
            Self::Py(time) => Ok(time),
            Self::Raw(time) => PyTime::new(
                py,
                time.hour,
                time.minute,
                time.second,
                time.microsecond,
                time_as_tzinfo(py, &time)?.as_ref(),
            ),
        }?;
        Ok(time.into())
    }
}

fn time_as_tzinfo<'py>(py: Python<'py>, time: &Time) -> PyResult<Option<Bound<'py, PyTzInfo>>> {
    match time.tz_offset {
        Some(offset) => {
            let tz_info: TzInfo = offset.try_into()?;
            Ok(Some(Bound::new(py, tz_info)?.into_any().downcast_into()?))
        }
        None => Ok(None),
    }
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum EitherDateTime<'a> {
    Raw(DateTime),
    Py(Bound<'a, PyDateTime>),
}

impl<'a> From<DateTime> for EitherDateTime<'a> {
    fn from(dt: DateTime) -> Self {
        Self::Raw(dt)
    }
}

impl<'a> From<Bound<'a, PyDateTime>> for EitherDateTime<'a> {
    fn from(dt: Bound<'a, PyDateTime>) -> Self {
        Self::Py(dt)
    }
}

pub fn pydatetime_as_datetime(py_dt: &Bound<'_, PyAny>) -> PyResult<DateTime> {
    Ok(DateTime {
        date: pydate_as_date(py_dt)?,
        time: pytime_as_time(py_dt, Some(py_dt))?,
    })
}

impl<'a> EitherDateTime<'a> {
    pub fn as_raw(&self) -> PyResult<DateTime> {
        match self {
            Self::Raw(dt) => Ok(dt.clone()),
            Self::Py(py_dt) => pydatetime_as_datetime(py_dt),
        }
    }

    pub fn try_into_pyobject(self, py: Python<'a>) -> PyResult<PyObject> {
        let dt = match self {
            Self::Raw(datetime) => PyDateTime::new(
                py,
                datetime.date.year.into(),
                datetime.date.month,
                datetime.date.day,
                datetime.time.hour,
                datetime.time.minute,
                datetime.time.second,
                datetime.time.microsecond,
                time_as_tzinfo(py, &datetime.time)?.as_ref(),
            )?,
            Self::Py(dt) => dt.clone(),
        };
        Ok(dt.into())
    }
}

pub fn bytes_as_date<'py>(input: &(impl Input<'py> + ?Sized), bytes: &[u8]) -> ValResult<EitherDate<'py>> {
    match Date::parse_bytes(bytes) {
        Ok(date) => Ok(date.into()),
        Err(err) => Err(ValError::new(
            ErrorType::DateParsing {
                error: Cow::Borrowed(err.get_documentation().unwrap_or_default()),
                context: None,
            },
            input,
        )),
    }
}

pub fn bytes_as_time<'py>(
    input: &(impl Input<'py> + ?Sized),
    bytes: &[u8],
    microseconds_overflow_behavior: MicrosecondsPrecisionOverflowBehavior,
) -> ValResult<EitherTime<'py>> {
    match Time::parse_bytes_with_config(
        bytes,
        &TimeConfig {
            microseconds_precision_overflow_behavior: microseconds_overflow_behavior,
            unix_timestamp_offset: Some(0),
        },
    ) {
        Ok(date) => Ok(date.into()),
        Err(err) => Err(ValError::new(
            ErrorType::TimeParsing {
                error: Cow::Borrowed(err.get_documentation().unwrap_or_default()),
                context: None,
            },
            input,
        )),
    }
}

pub fn bytes_as_datetime<'py>(
    input: &(impl Input<'py> + ?Sized),
    bytes: &[u8],
    microseconds_overflow_behavior: MicrosecondsPrecisionOverflowBehavior,
) -> ValResult<EitherDateTime<'py>> {
    match DateTime::parse_bytes_with_config(
        bytes,
        &TimeConfig {
            microseconds_precision_overflow_behavior: microseconds_overflow_behavior,
            unix_timestamp_offset: Some(0),
        },
    ) {
        Ok(dt) => Ok(dt.into()),
        Err(err) => Err(ValError::new(
            ErrorType::DatetimeParsing {
                error: Cow::Borrowed(err.get_documentation().unwrap_or_default()),
                context: None,
            },
            input,
        )),
    }
}

pub fn int_as_datetime<'py>(
    input: &(impl Input<'py> + ?Sized),
    timestamp: i64,
    timestamp_microseconds: u32,
) -> ValResult<EitherDateTime<'py>> {
    match DateTime::from_timestamp_with_config(
        timestamp,
        timestamp_microseconds,
        &TimeConfig {
            unix_timestamp_offset: Some(0),
            ..Default::default()
        },
    ) {
        Ok(dt) => Ok(dt.into()),
        Err(err) => Err(ValError::new(
            ErrorType::DatetimeParsing {
                error: Cow::Borrowed(err.get_documentation().unwrap_or_default()),
                context: None,
            },
            input,
        )),
    }
}

macro_rules! nan_check {
    ($input:ident, $float_value:ident, $error_type:ident) => {
        if $float_value.is_nan() {
            return Err(ValError::new(
                ErrorType::$error_type {
                    error: Cow::Borrowed("NaN values not permitted"),
                    context: None,
                },
                $input,
            ));
        }
    };
}

pub fn float_as_datetime<'py>(input: &(impl Input<'py> + ?Sized), timestamp: f64) -> ValResult<EitherDateTime<'py>> {
    nan_check!(input, timestamp, DatetimeParsing);
    let microseconds = timestamp.fract().abs() * 1_000_000.0;
    // checking for extra digits in microseconds is unreliable with large floats,
    // so we just round to the nearest microsecond
    int_as_datetime(input, timestamp.floor() as i64, microseconds.round() as u32)
}

pub fn date_as_datetime<'py>(date: &Bound<'py, PyDate>) -> PyResult<EitherDateTime<'py>> {
    let py = date.py();
    let dt = PyDateTime::new(
        py,
        date.getattr(intern!(py, "year"))?.extract()?,
        date.getattr(intern!(py, "month"))?.extract()?,
        date.getattr(intern!(py, "day"))?.extract()?,
        0,
        0,
        0,
        0,
        None,
    )?;
    Ok(dt.into())
}

const MAX_U32: i64 = u32::MAX as i64;

pub fn int_as_time<'py>(
    input: &(impl Input<'py> + ?Sized),
    timestamp: i64,
    timestamp_microseconds: u32,
) -> ValResult<EitherTime<'py>> {
    let time_timestamp: u32 = match timestamp {
        t if t < 0_i64 => {
            return Err(ValError::new(
                ErrorType::TimeParsing {
                    error: Cow::Borrowed("time in seconds should be positive"),
                    context: None,
                },
                input,
            ));
        }
        // continue and use the speedate error for >86400
        t if t > MAX_U32 => u32::MAX,
        // ok
        t => t as u32,
    };
    match Time::from_timestamp_with_config(
        time_timestamp,
        timestamp_microseconds,
        &TimeConfig {
            unix_timestamp_offset: Some(0),
            ..Default::default()
        },
    ) {
        Ok(dt) => Ok(dt.into()),
        Err(err) => Err(ValError::new(
            ErrorType::TimeParsing {
                error: Cow::Borrowed(err.get_documentation().unwrap_or_default()),
                context: None,
            },
            input,
        )),
    }
}

pub fn float_as_time<'py>(input: &(impl Input<'py> + ?Sized), timestamp: f64) -> ValResult<EitherTime<'py>> {
    nan_check!(input, timestamp, TimeParsing);
    let microseconds = timestamp.fract().abs() * 1_000_000.0;
    // round for same reason as above
    int_as_time(input, timestamp.floor() as i64, microseconds.round() as u32)
}

fn map_timedelta_err(input: impl ToErrorValue, err: ParseError) -> ValError {
    ValError::new(
        ErrorType::TimeDeltaParsing {
            error: Cow::Borrowed(err.get_documentation().unwrap_or_default()),
            context: None,
        },
        input,
    )
}

pub fn bytes_as_timedelta<'py>(
    input: &(impl Input<'py> + ?Sized),
    bytes: &[u8],
    microseconds_overflow_behavior: MicrosecondsPrecisionOverflowBehavior,
) -> ValResult<EitherTimedelta<'py>> {
    match Duration::parse_bytes_with_config(
        bytes,
        &TimeConfig {
            microseconds_precision_overflow_behavior: microseconds_overflow_behavior,
            unix_timestamp_offset: Some(0),
        },
    ) {
        Ok(dt) => Ok(dt.into()),
        Err(err) => Err(map_timedelta_err(input, err)),
    }
}

pub fn int_as_duration(input: impl ToErrorValue, total_seconds: i64) -> ValResult<Duration> {
    let positive = total_seconds >= 0;
    let total_seconds = total_seconds.unsigned_abs();
    // we can safely unwrap here since we've guaranteed seconds and microseconds can't cause overflow
    let days = (total_seconds / 86400) as u32;
    let seconds = (total_seconds % 86400) as u32;
    Duration::new(positive, days, seconds, 0).map_err(|err| map_timedelta_err(input, err))
}

pub fn float_as_duration(input: impl ToErrorValue, total_seconds: f64) -> ValResult<Duration> {
    nan_check!(input, total_seconds, TimeDeltaParsing);
    let positive = total_seconds >= 0_f64;
    let total_seconds = total_seconds.abs();
    let microsecond = total_seconds.fract() * 1_000_000.0;
    let days = (total_seconds / 86400f64) as u32;
    let seconds = total_seconds as u64 % 86400;
    Duration::new(positive, days, seconds as u32, microsecond.round() as u32)
        .map_err(|err| map_timedelta_err(input, err))
}

#[pyclass(module = "pydantic_core._pydantic_core", extends = PyTzInfo, frozen)]
#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct TzInfo {
    seconds: i32,
}

#[pymethods]
impl TzInfo {
    #[new]
    fn py_new(seconds: f32) -> PyResult<Self> {
        Self::try_from(seconds.trunc() as i32)
    }

    #[allow(unused_variables)]
    fn utcoffset<'py>(&self, py: Python<'py>, dt: &Bound<'_, PyAny>) -> PyResult<Bound<'py, PyDelta>> {
        PyDelta::new(py, 0, self.seconds, 0, true)
    }

    #[allow(unused_variables)]
    fn tzname(&self, dt: &Bound<'_, PyAny>) -> String {
        self.__str__()
    }

    #[allow(unused_variables)]
    fn dst(&self, dt: &Bound<'_, PyAny>) -> Option<&Bound<'_, PyDelta>> {
        None
    }

    fn fromutc<'py>(&self, dt: &Bound<'py, PyDateTime>) -> PyResult<Bound<'py, PyAny>> {
        let py = dt.py();
        dt.call_method1("__add__", (self.utcoffset(py, py.None().bind(py))?,))
    }

    fn __repr__(&self) -> String {
        format!("TzInfo({})", self.__str__())
    }

    fn __str__(&self) -> String {
        if self.seconds == 0 {
            return "UTC".to_string();
        }

        let (mins, seconds) = (self.seconds / 60, self.seconds % 60);
        let mut result = format!(
            "{}{:02}:{:02}",
            if self.seconds.signum() >= 0 { "+" } else { "-" },
            (mins / 60).abs(),
            (mins % 60).abs()
        );

        if seconds != 0 {
            result.push_str(&format!(":{:02}", seconds.abs()));
        }

        result
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.seconds.hash(&mut hasher);
        hasher.finish()
    }

    fn __richcmp__(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> PyResult<Py<PyAny>> {
        let py = other.py();
        if other.is_instance_of::<PyTzInfo>() {
            let offset_delta = other.call_method1(intern!(py, "utcoffset"), (py.None(),))?;
            if PyAnyMethods::is_none(&offset_delta) {
                return Ok(py.NotImplemented());
            }
            let offset_seconds: f64 = offset_delta.call_method0(intern!(py, "total_seconds"))?.extract()?;
            let offset = offset_seconds.round() as i32;
            Ok(op
                .matches(self.seconds.cmp(&offset))
                .into_pyobject(py)?
                .to_owned()
                .into())
        } else {
            Ok(py.NotImplemented())
        }
    }

    fn __deepcopy__(&self, py: Python, _memo: &Bound<'_, PyDict>) -> PyResult<Py<Self>> {
        Py::new(py, self.clone())
    }

    pub fn __reduce__<'py>(slf: &Bound<'py, Self>) -> Bound<'py, PyTuple> {
        let cls = slf.get_type();
        let args = (slf.get().seconds,);
        PyTuple::new((cls, args))
    }
}

impl TryFrom<i32> for TzInfo {
    type Error = PyErr;

    fn try_from(seconds: i32) -> PyResult<Self> {
        if seconds.abs() >= 86400 {
            Err(PyValueError::new_err(format!(
                "TzInfo offset must be strictly between -86400 and 86400 (24 hours) seconds, got {seconds}"
            )))
        } else {
            Ok(Self { seconds })
        }
    }
}
