use crate::errors::{ErrorKind, InputValue, LocItem, ValError, ValResult};

use super::datetime::{
    bytes_as_date, bytes_as_datetime, bytes_as_time, bytes_as_timedelta, float_as_datetime, float_as_duration,
    float_as_time, int_as_datetime, int_as_duration, int_as_time, EitherDate, EitherDateTime, EitherTime,
};
use super::shared::{float_as_int, int_as_bool, str_as_bool, str_as_int};
use super::{EitherBytes, EitherString, EitherTimedelta, GenericMapping, GenericSequence, Input, JsonInput};

impl<'a> Input<'a> for JsonInput {
    /// This is required by since JSON object keys are always strings, I don't think it can be called
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn as_loc_item(&'a self) -> LocItem {
        match self {
            JsonInput::Int(i) => LocItem::I(*i as usize),
            JsonInput::String(s) => s.as_str().into(),
            v => format!("{:?}", v).into(),
        }
    }

    fn as_error_value(&'a self) -> InputValue<'a> {
        InputValue::JsonInput(self)
    }

    fn is_none(&self) -> bool {
        matches!(self, JsonInput::Null)
    }

    fn strict_str<'data>(&'data self) -> ValResult<EitherString<'data>> {
        match self {
            JsonInput::String(s) => Ok(s.as_str().into()),
            _ => Err(ValError::new(ErrorKind::StrType, self)),
        }
    }

    fn lax_str<'data>(&'data self) -> ValResult<EitherString<'data>> {
        match self {
            JsonInput::String(s) => Ok(s.as_str().into()),
            JsonInput::Int(int) => Ok(int.to_string().into()),
            JsonInput::Float(float) => Ok(float.to_string().into()),
            _ => Err(ValError::new(ErrorKind::StrType, self)),
        }
    }

    fn validate_bool(&self, strict: bool) -> ValResult<bool> {
        if strict {
            match self {
                JsonInput::Bool(b) => Ok(*b),
                _ => Err(ValError::new(ErrorKind::BoolType, self)),
            }
        } else {
            match self {
                JsonInput::Bool(b) => Ok(*b),
                JsonInput::String(s) => str_as_bool(self, s),
                JsonInput::Int(int) => int_as_bool(self, *int),
                JsonInput::Float(float) => match float_as_int(self, *float) {
                    Ok(int) => int_as_bool(self, int),
                    _ => Err(ValError::new(ErrorKind::BoolType, self)),
                },
                _ => Err(ValError::new(ErrorKind::BoolType, self)),
            }
        }
    }

    fn strict_int(&self) -> ValResult<i64> {
        match self {
            JsonInput::Int(i) => Ok(*i),
            _ => Err(ValError::new(ErrorKind::IntType, self)),
        }
    }

    fn lax_int(&self) -> ValResult<i64> {
        match self {
            JsonInput::Bool(b) => match *b {
                true => Ok(1),
                false => Ok(0),
            },
            JsonInput::Int(i) => Ok(*i),
            JsonInput::Float(f) => float_as_int(self, *f),
            JsonInput::String(str) => str_as_int(self, str),
            _ => Err(ValError::new(ErrorKind::IntType, self)),
        }
    }

    fn validate_float(&self, strict: bool) -> ValResult<f64> {
        if strict {
            match self {
                JsonInput::Float(f) => Ok(*f),
                JsonInput::Int(i) => Ok(*i as f64),
                _ => Err(ValError::new(ErrorKind::FloatType, self)),
            }
        } else {
            match self {
                JsonInput::Bool(b) => match *b {
                    true => Ok(1.0),
                    false => Ok(0.0),
                },
                JsonInput::Float(f) => Ok(*f),
                JsonInput::Int(i) => Ok(*i as f64),
                JsonInput::String(str) => match str.parse() {
                    Ok(i) => Ok(i),
                    Err(_) => Err(ValError::new(ErrorKind::FloatParsing, self)),
                },
                _ => Err(ValError::new(ErrorKind::FloatType, self)),
            }
        }
    }

    fn validate_dict<'data>(&'data self, _strict: bool) -> ValResult<GenericMapping<'data>> {
        match self {
            JsonInput::Object(dict) => Ok(dict.into()),
            _ => Err(ValError::new(ErrorKind::DictType, self)),
        }
    }

    fn validate_list<'data>(&'data self, _strict: bool) -> ValResult<GenericSequence<'data>> {
        match self {
            JsonInput::Array(a) => Ok(a.into()),
            _ => Err(ValError::new(ErrorKind::ListType, self)),
        }
    }

    fn validate_set<'data>(&'data self, _strict: bool) -> ValResult<GenericSequence<'data>> {
        // we allow a list here since otherwise it would be impossible to create a set from JSON
        match self {
            JsonInput::Array(a) => Ok(a.into()),
            _ => Err(ValError::new(ErrorKind::SetType, self)),
        }
    }

    fn validate_frozenset<'data>(&'data self, _strict: bool) -> ValResult<GenericSequence<'data>> {
        match self {
            JsonInput::Array(a) => Ok(a.into()),
            _ => Err(ValError::new(ErrorKind::FrozenSetType, self)),
        }
    }

    fn validate_bytes<'data>(&'data self, _strict: bool) -> ValResult<EitherBytes<'data>> {
        match self {
            JsonInput::String(s) => Ok(s.as_bytes().into()),
            _ => Err(ValError::new(ErrorKind::BytesType, self)),
        }
    }

    fn validate_date(&self, _strict: bool) -> ValResult<EitherDate> {
        match self {
            JsonInput::String(v) => bytes_as_date(self, v.as_bytes()),
            _ => Err(ValError::new(ErrorKind::DateType, self)),
        }
        // NO custom `lax_date` implementation, if strict_date fails, the validator will fallback to lax_datetime
        // then check there's no remainder
    }

    fn validate_time(&self, strict: bool) -> ValResult<EitherTime> {
        if strict {
            match self {
                JsonInput::String(v) => bytes_as_time(self, v.as_bytes()),
                _ => Err(ValError::new(ErrorKind::TimeType, self)),
            }
        } else {
            match self {
                JsonInput::String(v) => bytes_as_time(self, v.as_bytes()),
                JsonInput::Int(v) => int_as_time(self, *v, 0),
                JsonInput::Float(v) => float_as_time(self, *v),
                _ => Err(ValError::new(ErrorKind::TimeType, self)),
            }
        }
    }

    fn validate_datetime(&self, strict: bool) -> ValResult<EitherDateTime> {
        if strict {
            match self {
                JsonInput::String(v) => bytes_as_datetime(self, v.as_bytes()),
                _ => Err(ValError::new(ErrorKind::DateTimeType, self)),
            }
        } else {
            match self {
                JsonInput::String(v) => bytes_as_datetime(self, v.as_bytes()),
                JsonInput::Int(v) => int_as_datetime(self, *v, 0),
                JsonInput::Float(v) => float_as_datetime(self, *v),
                _ => Err(ValError::new(ErrorKind::DateTimeType, self)),
            }
        }
    }

    fn validate_tuple<'data>(&'data self, _strict: bool) -> ValResult<GenericSequence<'data>> {
        // just as in set's case, List has to be allowed
        match self {
            JsonInput::Array(a) => Ok(a.into()),
            _ => Err(ValError::new(ErrorKind::TupleType, self)),
        }
    }

    fn validate_timedelta(&self, strict: bool) -> ValResult<EitherTimedelta> {
        if strict {
            match self {
                JsonInput::String(v) => bytes_as_timedelta(self, v.as_bytes()),
                _ => Err(ValError::new(ErrorKind::TimeDeltaType, self)),
            }
        } else {
            match self {
                JsonInput::String(v) => bytes_as_timedelta(self, v.as_bytes()),
                JsonInput::Int(v) => Ok(int_as_duration(*v).into()),
                JsonInput::Float(v) => Ok(float_as_duration(*v).into()),
                _ => Err(ValError::new(ErrorKind::TimeDeltaType, self)),
            }
        }
    }
}

/// Required for Dict keys so the string can behave like an Input
impl<'a> Input<'a> for String {
    fn as_loc_item(&'a self) -> LocItem {
        self.to_string().into()
    }

    fn as_error_value(&'a self) -> InputValue<'a> {
        InputValue::String(self)
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn is_none(&self) -> bool {
        false
    }

    fn validate_str<'data>(&'data self, _strict: bool) -> ValResult<EitherString<'data>> {
        Ok(self.as_str().into())
    }
    fn lax_str<'data>(&'data self) -> ValResult<EitherString<'data>> {
        Ok(self.as_str().into())
    }
    fn strict_str<'data>(&'data self) -> ValResult<EitherString<'data>> {
        Ok(self.as_str().into())
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn validate_bool(&self, strict: bool) -> ValResult<bool> {
        if strict {
            Err(ValError::new(ErrorKind::BoolType, self))
        } else {
            str_as_bool(self, self)
        }
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_int(&self) -> ValResult<i64> {
        Err(ValError::new(ErrorKind::IntType, self))
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn lax_int(&self) -> ValResult<i64> {
        match self.parse() {
            Ok(i) => Ok(i),
            Err(_) => Err(ValError::new(ErrorKind::IntParsing, self)),
        }
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn validate_float(&self, strict: bool) -> ValResult<f64> {
        if strict {
            Err(ValError::new(ErrorKind::FloatType, self))
        } else {
            match self.parse() {
                Ok(i) => Ok(i),
                Err(_) => Err(ValError::new(ErrorKind::FloatParsing, self)),
            }
        }
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn validate_dict<'data>(&'data self, _strict: bool) -> ValResult<GenericMapping<'data>> {
        Err(ValError::new(ErrorKind::DictType, self))
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn validate_list<'data>(&'data self, _strict: bool) -> ValResult<GenericSequence<'data>> {
        Err(ValError::new(ErrorKind::ListType, self))
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn validate_set<'data>(&'data self, _strict: bool) -> ValResult<GenericSequence<'data>> {
        Err(ValError::new(ErrorKind::SetType, self))
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn validate_frozenset<'data>(&'data self, _strict: bool) -> ValResult<GenericSequence<'data>> {
        Err(ValError::new(ErrorKind::FrozenSetType, self))
    }

    fn validate_bytes<'data>(&'data self, _strict: bool) -> ValResult<EitherBytes<'data>> {
        Ok(self.as_bytes().into())
    }

    fn validate_date(&self, _strict: bool) -> ValResult<EitherDate> {
        bytes_as_date(self, self.as_bytes())
    }

    fn validate_time(&self, _strict: bool) -> ValResult<EitherTime> {
        bytes_as_time(self, self.as_bytes())
    }

    fn validate_datetime(&self, _strict: bool) -> ValResult<EitherDateTime> {
        bytes_as_datetime(self, self.as_bytes())
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn validate_tuple<'data>(&'data self, _strict: bool) -> ValResult<GenericSequence<'data>> {
        Err(ValError::new(ErrorKind::TupleType, self))
    }

    fn validate_timedelta(&self, _strict: bool) -> ValResult<EitherTimedelta> {
        bytes_as_timedelta(self, self.as_bytes())
    }
}
