use crate::errors::{ErrorKind, InputValue, LocItem, ValError, ValResult};
use pyo3::types::PyString;
use pyo3::Python;

use super::datetime::{
    bytes_as_date, bytes_as_datetime, bytes_as_time, bytes_as_timedelta, float_as_datetime, float_as_duration,
    float_as_time, int_as_datetime, int_as_duration, int_as_time, EitherDate, EitherDateTime, EitherTime,
};
use super::shared::{float_as_int, int_as_bool, str_as_bool, str_as_int};
use super::{
    EitherBytes, EitherTimedelta, GenericArguments, GenericListLike, GenericMapping, Input, JsonArgs, JsonInput,
};

impl<'a> Input<'a> for JsonInput {
    /// This is required by since JSON object keys are always strings, I don't think it can be called
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn as_loc_item(&self, py: Python) -> LocItem {
        match self {
            JsonInput::Int(i) => LocItem::I(*i as usize),
            JsonInput::String(s) => s.as_ref(py).to_string().into(),
            v => format!("{:?}", v).into(),
        }
    }

    fn as_error_value(&'a self) -> InputValue<'a> {
        InputValue::JsonInput(self)
    }

    fn is_none(&self) -> bool {
        matches!(self, JsonInput::Null)
    }

    fn validate_args(&'a self) -> ValResult<'a, GenericArguments<'a>> {
        match self {
            JsonInput::Object(kwargs) => Ok(JsonArgs::new(None, Some(kwargs)).into()),
            JsonInput::Array(array) => {
                if array.len() != 2 {
                    Err(ValError::new(ErrorKind::ArgumentsType, self))
                } else {
                    let args = match unsafe { array.get_unchecked(0) } {
                        JsonInput::Null => None,
                        JsonInput::Array(args) => Some(args.as_slice()),
                        _ => return Err(ValError::new(ErrorKind::ArgumentsType, self)),
                    };
                    let kwargs = match unsafe { array.get_unchecked(1) } {
                        JsonInput::Null => None,
                        JsonInput::Object(kwargs) => Some(kwargs),
                        _ => return Err(ValError::new(ErrorKind::ArgumentsType, self)),
                    };
                    Ok(JsonArgs::new(args, kwargs).into())
                }
            }
            _ => Err(ValError::new(ErrorKind::ArgumentsType, self)),
        }
    }

    fn strict_str(&'a self, py: Python<'a>) -> ValResult<&'a PyString> {
        match self {
            JsonInput::String(s) => Ok(s.as_ref(py)),
            _ => Err(ValError::new(ErrorKind::StrType, self)),
        }
    }
    fn lax_str(&'a self, py: Python<'a>) -> ValResult<&'a PyString> {
        match self {
            JsonInput::String(s) => Ok(s.as_ref(py)),
            JsonInput::Int(int) => Ok(PyString::new(py, &int.to_string())),
            JsonInput::Float(float) => Ok(PyString::new(py, &float.to_string())),
            _ => Err(ValError::new(ErrorKind::StrType, self)),
        }
    }

    fn validate_bytes(&'a self, py: Python, _strict: bool) -> ValResult<EitherBytes<'a>> {
        match self {
            JsonInput::String(s) => {
                let string = s.as_ref(py).to_string_lossy().to_string();
                Ok(string.into_bytes().into())
            }
            _ => Err(ValError::new(ErrorKind::BytesType, self)),
        }
    }
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_bytes(&'a self, py: Python) -> ValResult<EitherBytes<'a>> {
        self.validate_bytes(py, false)
    }

    fn strict_bool(&self) -> ValResult<bool> {
        match self {
            JsonInput::Bool(b) => Ok(*b),
            _ => Err(ValError::new(ErrorKind::BoolType, self)),
        }
    }
    fn lax_bool(&self, py: Python) -> ValResult<bool> {
        match self {
            JsonInput::Bool(b) => Ok(*b),
            JsonInput::String(s) => str_as_bool(self, &s.as_ref(py).to_string_lossy()),
            JsonInput::Int(int) => int_as_bool(self, *int),
            JsonInput::Float(float) => match float_as_int(self, *float) {
                Ok(int) => int_as_bool(self, int),
                _ => Err(ValError::new(ErrorKind::BoolType, self)),
            },
            _ => Err(ValError::new(ErrorKind::BoolType, self)),
        }
    }

    fn strict_int(&self) -> ValResult<i64> {
        match self {
            JsonInput::Int(i) => Ok(*i),
            _ => Err(ValError::new(ErrorKind::IntType, self)),
        }
    }
    fn lax_int(&self, py: Python) -> ValResult<i64> {
        match self {
            JsonInput::Bool(b) => match *b {
                true => Ok(1),
                false => Ok(0),
            },
            JsonInput::Int(i) => Ok(*i),
            JsonInput::Float(f) => float_as_int(self, *f),
            JsonInput::String(str) => str_as_int(self, &str.as_ref(py).to_string_lossy()),
            _ => Err(ValError::new(ErrorKind::IntType, self)),
        }
    }

    fn strict_float(&self) -> ValResult<f64> {
        match self {
            JsonInput::Float(f) => Ok(*f),
            JsonInput::Int(i) => Ok(*i as f64),
            _ => Err(ValError::new(ErrorKind::FloatType, self)),
        }
    }
    fn lax_float(&self, py: Python) -> ValResult<f64> {
        match self {
            JsonInput::Bool(b) => match *b {
                true => Ok(1.0),
                false => Ok(0.0),
            },
            JsonInput::Float(f) => Ok(*f),
            JsonInput::Int(i) => Ok(*i as f64),
            JsonInput::String(str) => match str.as_ref(py).to_string_lossy().as_ref().parse() {
                Ok(i) => Ok(i),
                Err(_) => Err(ValError::new(ErrorKind::FloatParsing, self)),
            },
            _ => Err(ValError::new(ErrorKind::FloatType, self)),
        }
    }

    fn validate_dict(&'a self, _strict: bool) -> ValResult<GenericMapping<'a>> {
        match self {
            JsonInput::Object(dict) => Ok(dict.into()),
            _ => Err(ValError::new(ErrorKind::DictType, self)),
        }
    }
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_dict(&'a self) -> ValResult<GenericMapping<'a>> {
        self.validate_dict(false)
    }

    fn validate_list(&'a self, _strict: bool) -> ValResult<GenericListLike<'a>> {
        match self {
            JsonInput::Array(a) => Ok(a.into()),
            _ => Err(ValError::new(ErrorKind::ListType, self)),
        }
    }
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_list(&'a self) -> ValResult<GenericListLike<'a>> {
        self.validate_list(false)
    }

    fn validate_tuple(&'a self, _strict: bool) -> ValResult<GenericListLike<'a>> {
        // just as in set's case, List has to be allowed
        match self {
            JsonInput::Array(a) => Ok(a.into()),
            _ => Err(ValError::new(ErrorKind::TupleType, self)),
        }
    }
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_tuple(&'a self) -> ValResult<GenericListLike<'a>> {
        self.validate_tuple(false)
    }

    fn validate_set(&'a self, _strict: bool) -> ValResult<GenericListLike<'a>> {
        // we allow a list here since otherwise it would be impossible to create a set from JSON
        match self {
            JsonInput::Array(a) => Ok(a.into()),
            _ => Err(ValError::new(ErrorKind::SetType, self)),
        }
    }
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_set(&'a self) -> ValResult<GenericListLike<'a>> {
        self.validate_set(false)
    }

    fn validate_frozenset(&'a self, _strict: bool) -> ValResult<GenericListLike<'a>> {
        // we allow a list here since otherwise it would be impossible to create a frozenset from JSON
        match self {
            JsonInput::Array(a) => Ok(a.into()),
            _ => Err(ValError::new(ErrorKind::FrozenSetType, self)),
        }
    }
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_frozenset(&'a self) -> ValResult<GenericListLike<'a>> {
        self.validate_frozenset(false)
    }

    fn validate_date(&self, py: Python, _strict: bool) -> ValResult<EitherDate> {
        match self {
            JsonInput::String(v) => bytes_as_date(self, v.as_ref(py).to_string_lossy().as_bytes()),
            _ => Err(ValError::new(ErrorKind::DateType, self)),
        }
    }
    // NO custom `lax_date` implementation, if strict_date fails, the validator will fallback to lax_datetime
    // then check there's no remainder
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_date(&self, py: Python) -> ValResult<EitherDate> {
        self.validate_date(py, false)
    }

    fn strict_time(&self, py: Python) -> ValResult<EitherTime> {
        match self {
            JsonInput::String(v) => bytes_as_time(self, v.as_ref(py).to_string_lossy().as_bytes()),
            _ => Err(ValError::new(ErrorKind::TimeType, self)),
        }
    }
    fn lax_time(&self, py: Python) -> ValResult<EitherTime> {
        match self {
            JsonInput::String(v) => bytes_as_time(self, v.as_ref(py).to_string_lossy().as_bytes()),
            JsonInput::Int(v) => int_as_time(self, *v, 0),
            JsonInput::Float(v) => float_as_time(self, *v),
            _ => Err(ValError::new(ErrorKind::TimeType, self)),
        }
    }

    fn strict_datetime(&self, py: Python) -> ValResult<EitherDateTime> {
        match self {
            JsonInput::String(v) => bytes_as_datetime(self, v.as_ref(py).to_string_lossy().as_bytes()),
            _ => Err(ValError::new(ErrorKind::DateTimeType, self)),
        }
    }
    fn lax_datetime(&self, py: Python) -> ValResult<EitherDateTime> {
        match self {
            JsonInput::String(v) => bytes_as_datetime(self, v.as_ref(py).to_string_lossy().as_bytes()),
            JsonInput::Int(v) => int_as_datetime(self, *v, 0),
            JsonInput::Float(v) => float_as_datetime(self, *v),
            _ => Err(ValError::new(ErrorKind::DateTimeType, self)),
        }
    }

    fn strict_timedelta(&self, py: Python) -> ValResult<EitherTimedelta> {
        match self {
            JsonInput::String(v) => bytes_as_timedelta(self, v.as_ref(py).to_string_lossy().as_bytes()),
            _ => Err(ValError::new(ErrorKind::TimeDeltaType, self)),
        }
    }
    fn lax_timedelta(&self, py: Python) -> ValResult<EitherTimedelta> {
        match self {
            JsonInput::String(v) => bytes_as_timedelta(self, v.as_ref(py).to_string_lossy().as_bytes()),
            JsonInput::Int(v) => Ok(int_as_duration(*v).into()),
            JsonInput::Float(v) => Ok(float_as_duration(*v).into()),
            _ => Err(ValError::new(ErrorKind::TimeDeltaType, self)),
        }
    }
}

/// Required for Dict keys so the string can behave like an Input
impl<'a> Input<'a> for String {
    fn as_loc_item(&self, _py: Python) -> LocItem {
        self.to_string().into()
    }

    fn as_error_value(&'a self) -> InputValue<'a> {
        InputValue::String(self)
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn is_none(&self) -> bool {
        false
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn validate_args(&'a self) -> ValResult<'a, GenericArguments<'a>> {
        Err(ValError::new(ErrorKind::ArgumentsType, self))
    }

    fn validate_str(&'a self, py: Python<'a>, _strict: bool) -> ValResult<&'a PyString> {
        Ok(PyString::new(py, self))
    }
    fn strict_str(&'a self, py: Python<'a>) -> ValResult<&'a PyString> {
        self.validate_str(py, false)
    }

    fn validate_bytes(&'a self, _py: Python, _strict: bool) -> ValResult<EitherBytes<'a>> {
        Ok(self.as_bytes().into())
    }
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_bytes(&'a self, py: Python) -> ValResult<EitherBytes<'a>> {
        self.validate_bytes(py, false)
    }

    fn strict_bool(&self) -> ValResult<bool> {
        Err(ValError::new(ErrorKind::BoolType, self))
    }
    fn lax_bool(&self, _py: Python) -> ValResult<bool> {
        str_as_bool(self, self)
    }

    fn strict_int(&self) -> ValResult<i64> {
        Err(ValError::new(ErrorKind::IntType, self))
    }
    fn lax_int(&self, _py: Python) -> ValResult<i64> {
        match self.parse() {
            Ok(i) => Ok(i),
            Err(_) => Err(ValError::new(ErrorKind::IntParsing, self)),
        }
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_float(&self) -> ValResult<f64> {
        Err(ValError::new(ErrorKind::FloatType, self))
    }
    fn lax_float(&self, _py: Python) -> ValResult<f64> {
        match self.parse() {
            Ok(i) => Ok(i),
            Err(_) => Err(ValError::new(ErrorKind::FloatParsing, self)),
        }
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn validate_dict(&'a self, _strict: bool) -> ValResult<GenericMapping<'a>> {
        Err(ValError::new(ErrorKind::DictType, self))
    }
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_dict(&'a self) -> ValResult<GenericMapping<'a>> {
        self.validate_dict(false)
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn validate_list(&'a self, _strict: bool) -> ValResult<GenericListLike<'a>> {
        Err(ValError::new(ErrorKind::ListType, self))
    }
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_list(&'a self) -> ValResult<GenericListLike<'a>> {
        self.validate_list(false)
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn validate_tuple(&'a self, _strict: bool) -> ValResult<GenericListLike<'a>> {
        Err(ValError::new(ErrorKind::TupleType, self))
    }
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_tuple(&'a self) -> ValResult<GenericListLike<'a>> {
        self.validate_tuple(false)
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn validate_set(&'a self, _strict: bool) -> ValResult<GenericListLike<'a>> {
        Err(ValError::new(ErrorKind::SetType, self))
    }
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_set(&'a self) -> ValResult<GenericListLike<'a>> {
        self.validate_set(false)
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn validate_frozenset(&'a self, _strict: bool) -> ValResult<GenericListLike<'a>> {
        Err(ValError::new(ErrorKind::FrozenSetType, self))
    }
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_frozenset(&'a self) -> ValResult<GenericListLike<'a>> {
        self.validate_frozenset(false)
    }

    fn validate_date(&self, _py: Python, _strict: bool) -> ValResult<EitherDate> {
        bytes_as_date(self, self.as_bytes())
    }
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_date(&self, py: Python) -> ValResult<EitherDate> {
        self.validate_date(py, false)
    }

    fn validate_time(&self, _py: Python, _strict: bool) -> ValResult<EitherTime> {
        bytes_as_time(self, self.as_bytes())
    }
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_time(&self, py: Python) -> ValResult<EitherTime> {
        self.validate_time(py, false)
    }

    fn validate_datetime(&self, _py: Python, _strict: bool) -> ValResult<EitherDateTime> {
        bytes_as_datetime(self, self.as_bytes())
    }
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_datetime(&self, py: Python) -> ValResult<EitherDateTime> {
        self.validate_datetime(py, false)
    }

    fn validate_timedelta(&self, _py: Python, _strict: bool) -> ValResult<EitherTimedelta> {
        bytes_as_timedelta(self, self.as_bytes())
    }
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_timedelta(&self, py: Python) -> ValResult<EitherTimedelta> {
        self.validate_timedelta(py, false)
    }
}
