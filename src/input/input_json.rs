use pyo3::types::PyType;

use crate::errors::location::LocItem;
use crate::errors::{err_val_error, ErrorKind, InputValue, ValResult};

use super::datetime::{
    bytes_as_date, bytes_as_datetime, bytes_as_time, float_as_datetime, float_as_time, int_as_datetime, int_as_time,
    EitherDate, EitherDateTime, EitherTime,
};
use super::shared::{float_as_int, int_as_bool, str_as_bool, str_as_int};
use super::{EitherBytes, EitherString, GenericMapping, GenericSequence, Input, JsonInput};

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
            _ => Err(err_val_error(ErrorKind::StrType, self, None)),
        }
    }

    fn lax_str<'data>(&'data self) -> ValResult<EitherString<'data>> {
        match self {
            JsonInput::String(s) => Ok(s.as_str().into()),
            JsonInput::Int(int) => Ok(int.to_string().into()),
            JsonInput::Float(float) => Ok(float.to_string().into()),
            _ => Err(err_val_error(ErrorKind::StrType, self, None)),
        }
    }

    fn strict_bool(&self) -> ValResult<bool> {
        match self {
            JsonInput::Bool(b) => Ok(*b),
            _ => Err(err_val_error(ErrorKind::BoolType, self, None)),
        }
    }

    fn lax_bool(&self) -> ValResult<bool> {
        match self {
            JsonInput::Bool(b) => Ok(*b),
            JsonInput::String(s) => str_as_bool(self, s),
            JsonInput::Int(int) => int_as_bool(self, *int),
            // TODO float??
            _ => Err(err_val_error(ErrorKind::BoolType, self, None)),
        }
    }

    fn strict_int(&self) -> ValResult<i64> {
        match self {
            JsonInput::Int(i) => Ok(*i),
            _ => Err(err_val_error(ErrorKind::IntType, self, None)),
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
            _ => Err(err_val_error(ErrorKind::IntType, self, None)),
        }
    }

    fn strict_float(&self) -> ValResult<f64> {
        match self {
            JsonInput::Float(f) => Ok(*f),
            JsonInput::Int(i) => Ok(*i as f64),
            _ => Err(err_val_error(ErrorKind::FloatType, self, None)),
        }
    }

    fn lax_float(&self) -> ValResult<f64> {
        match self {
            JsonInput::Bool(b) => match *b {
                true => Ok(1.0),
                false => Ok(0.0),
            },
            JsonInput::Float(f) => Ok(*f),
            JsonInput::Int(i) => Ok(*i as f64),
            JsonInput::String(str) => match str.parse() {
                Ok(i) => Ok(i),
                Err(_) => Err(err_val_error(ErrorKind::FloatParsing, self, None)),
            },
            _ => Err(err_val_error(ErrorKind::FloatType, self, None)),
        }
    }

    fn strict_model_check(&self, _class: &PyType) -> ValResult<bool> {
        Ok(false)
    }

    fn strict_dict<'data>(&'data self) -> ValResult<GenericMapping<'data>> {
        match self {
            JsonInput::Object(dict) => Ok(dict.into()),
            _ => Err(err_val_error(ErrorKind::DictType, self, None)),
        }
    }

    fn strict_list<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        match self {
            JsonInput::Array(a) => Ok(a.into()),
            _ => Err(err_val_error(ErrorKind::ListType, self, None)),
        }
    }

    fn strict_set<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        // we allow a list here since otherwise it would be impossible to create a set from JSON
        match self {
            JsonInput::Array(a) => Ok(a.into()),
            _ => Err(err_val_error(ErrorKind::SetType, self, None)),
        }
    }

    fn strict_frozenset<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        match self {
            JsonInput::Array(a) => Ok(a.into()),
            _ => Err(err_val_error(ErrorKind::FrozenSetType, self, None)),
        }
    }

    fn strict_bytes<'data>(&'data self) -> ValResult<EitherBytes<'data>> {
        match self {
            JsonInput::String(s) => Ok(s.as_bytes().into()),
            _ => Err(err_val_error(ErrorKind::BytesType, self, None)),
        }
    }

    fn strict_date(&self) -> ValResult<EitherDate> {
        match self {
            JsonInput::String(v) => bytes_as_date(self, v.as_bytes()),
            _ => Err(err_val_error(ErrorKind::DateType, self, None)),
        }
    }

    fn strict_time(&self) -> ValResult<EitherTime> {
        match self {
            JsonInput::String(v) => bytes_as_time(self, v.as_bytes()),
            _ => Err(err_val_error(ErrorKind::TimeType, self, None)),
        }
    }

    // NO custom `lax_date` implementation, if strict_date fails, the validator will fallback to lax_datetime
    // then check there's no remainder

    fn lax_time(&self) -> ValResult<EitherTime> {
        match self {
            JsonInput::String(v) => bytes_as_time(self, v.as_bytes()),
            JsonInput::Int(v) => int_as_time(self, *v, 0),
            JsonInput::Float(v) => float_as_time(self, *v),
            _ => Err(err_val_error(ErrorKind::TimeType, self, None)),
        }
    }

    fn strict_datetime(&self) -> ValResult<EitherDateTime> {
        match self {
            JsonInput::String(v) => bytes_as_datetime(self, v.as_bytes()),
            _ => Err(err_val_error(ErrorKind::DateTimeType, self, None)),
        }
    }

    fn lax_datetime(&self) -> ValResult<EitherDateTime> {
        match self {
            JsonInput::String(v) => bytes_as_datetime(self, v.as_bytes()),
            JsonInput::Int(v) => int_as_datetime(self, *v, 0),
            JsonInput::Float(v) => float_as_datetime(self, *v),
            _ => Err(err_val_error(ErrorKind::DateTimeType, self, None)),
        }
    }

    fn strict_tuple<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        // just as in set's case, List has to be allowed
        match self {
            JsonInput::Array(a) => Ok(a.into()),
            _ => Err(err_val_error(ErrorKind::TupleType, self, None)),
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

    fn strict_str<'data>(&'data self) -> ValResult<EitherString<'data>> {
        Ok(self.as_str().into())
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_bool(&self) -> ValResult<bool> {
        Err(err_val_error(ErrorKind::BoolType, self, None))
    }

    fn lax_bool(&self) -> ValResult<bool> {
        str_as_bool(self, self)
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_int(&self) -> ValResult<i64> {
        Err(err_val_error(ErrorKind::IntType, self, None))
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn lax_int(&self) -> ValResult<i64> {
        match self.parse() {
            Ok(i) => Ok(i),
            Err(_) => Err(err_val_error(ErrorKind::IntParsing, self, None)),
        }
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_float(&self) -> ValResult<f64> {
        Err(err_val_error(ErrorKind::FloatType, self, None))
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn lax_float(&self) -> ValResult<f64> {
        match self.parse() {
            Ok(i) => Ok(i),
            Err(_) => Err(err_val_error(ErrorKind::FloatParsing, self, None)),
        }
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_model_check(&self, _class: &PyType) -> ValResult<bool> {
        Ok(false)
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_dict<'data>(&'data self) -> ValResult<GenericMapping<'data>> {
        Err(err_val_error(ErrorKind::DictType, self, None))
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_list<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        Err(err_val_error(ErrorKind::ListType, self, None))
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_set<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        Err(err_val_error(ErrorKind::SetType, self, None))
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_frozenset<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        Err(err_val_error(ErrorKind::FrozenSetType, self, None))
    }

    fn strict_bytes<'data>(&'data self) -> ValResult<EitherBytes<'data>> {
        Ok(self.as_bytes().into())
    }

    fn strict_date(&self) -> ValResult<EitherDate> {
        bytes_as_date(self, self.as_bytes())
    }

    fn strict_time(&self) -> ValResult<EitherTime> {
        bytes_as_time(self, self.as_bytes())
    }

    fn strict_datetime(&self) -> ValResult<EitherDateTime> {
        bytes_as_datetime(self, self.as_bytes())
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn strict_tuple<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        Err(err_val_error(ErrorKind::TupleType, self, None))
    }
}
