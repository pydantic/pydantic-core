use std::fmt;

use pyo3::prelude::*;
use pyo3::types::PyType;

use crate::errors::{InputValue, LocItem, ValResult};
use crate::input::datetime::EitherTime;

use super::datetime::{EitherDate, EitherDateTime, EitherTimedelta};
use super::return_enums::{EitherBytes, EitherString};
use super::{GenericMapping, GenericSequence};

pub trait Input<'a>: fmt::Debug + ToPyObject {
    fn as_loc_item(&'a self) -> LocItem;

    fn as_error_value(&'a self) -> InputValue<'a>;

    fn identity(&'a self) -> Option<usize> {
        None
    }

    fn is_none(&self) -> bool;

    fn validate_str<'data>(&'data self, strict: bool) -> ValResult<EitherString<'data>>;

    fn validate_bool(&self, strict: bool) -> ValResult<bool>;

    fn validate_int(&self, strict: bool) -> ValResult<i64>;

    fn validate_float(&self, strict: bool) -> ValResult<f64>;

    fn is_type(&self, _class: &PyType) -> ValResult<bool> {
        Ok(false)
    }

    fn validate_dict<'data>(&'data self, strict: bool) -> ValResult<GenericMapping<'data>>;

    fn typed_dict<'data>(&'data self, _from_attributes: bool, from_mapping: bool) -> ValResult<GenericMapping<'data>> {
        self.validate_dict(!from_mapping)
    }

    fn validate_list<'data>(&'data self, strict: bool) -> ValResult<GenericSequence<'data>>;

    fn validate_set<'data>(&'data self, strict: bool) -> ValResult<GenericSequence<'data>>;

    fn validate_frozenset<'data>(&'data self, strict: bool) -> ValResult<GenericSequence<'data>>;

    fn validate_bytes<'data>(&'data self, strict: bool) -> ValResult<EitherBytes<'data>>;

    fn validate_date(&self, strict: bool) -> ValResult<EitherDate>;

    fn validate_time(&self, strict: bool) -> ValResult<EitherTime>;

    fn validate_datetime(&self, strict: bool) -> ValResult<EitherDateTime>;

    fn validate_tuple<'data>(&'data self, strict: bool) -> ValResult<GenericSequence<'data>>;

    fn validate_timedelta(&self, strict: bool) -> ValResult<EitherTimedelta>;

    fn is_instance(&self, _class: &PyType) -> PyResult<bool> {
        Ok(false)
    }

    fn callable(&self) -> bool {
        false
    }
}
