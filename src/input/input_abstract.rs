use std::fmt;

use pyo3::prelude::*;
use pyo3::types::PyType;

use crate::errors::{InputValue, LocItem, ValResult};
use crate::input::datetime::EitherTime;

use super::datetime::{EitherDate, EitherDateTime, EitherTimedelta};
use super::return_enums::{EitherBytes, EitherString};
use super::{GenericMapping, GenericSequence};

/// all types have three methods: `validate_*`, `strict_*`, `lax_*`
/// the convention is to either implement:
/// * `strict_*` & `lax_*` if they have different behaviour
/// * or, `validate_*` and `strict_*` to just call `validate_*`
pub trait Input<'a>: fmt::Debug + ToPyObject {
    fn as_loc_item(&'a self) -> LocItem;

    fn as_error_value(&'a self) -> InputValue<'a>;

    fn identity(&'a self) -> Option<usize> {
        None
    }

    fn is_none(&self) -> bool;

    fn is_type(&self, _class: &PyType) -> ValResult<bool> {
        Ok(false)
    }

    fn is_instance(&self, _class: &PyType) -> PyResult<bool> {
        Ok(false)
    }

    fn callable(&self) -> bool {
        false
    }

    fn validate_str<'data>(&'data self, strict: bool) -> ValResult<EitherString<'data>> {
        if strict {
            self.strict_str()
        } else {
            self.lax_str()
        }
    }
    fn strict_str<'data>(&'data self) -> ValResult<EitherString<'data>>;
    fn lax_str<'data>(&'data self) -> ValResult<EitherString<'data>> {
        self.strict_str()
    }

    fn validate_bytes<'data>(&'data self, strict: bool) -> ValResult<EitherBytes<'data>> {
        if strict {
            self.strict_bytes()
        } else {
            self.lax_bytes()
        }
    }
    fn strict_bytes<'data>(&'data self) -> ValResult<EitherBytes<'data>>;
    fn lax_bytes<'data>(&'data self) -> ValResult<EitherBytes<'data>> {
        self.strict_bytes()
    }

    fn validate_bool(&self, strict: bool) -> ValResult<bool> {
        if strict {
            self.strict_bool()
        } else {
            self.lax_bool()
        }
    }
    fn strict_bool(&self) -> ValResult<bool>;
    fn lax_bool(&self) -> ValResult<bool> {
        self.strict_bool()
    }

    fn validate_int(&self, strict: bool) -> ValResult<i64> {
        if strict {
            self.strict_int()
        } else {
            self.lax_int()
        }
    }
    fn strict_int(&self) -> ValResult<i64>;
    fn lax_int(&self) -> ValResult<i64> {
        self.strict_int()
    }

    fn validate_float(&self, strict: bool) -> ValResult<f64> {
        if strict {
            self.strict_float()
        } else {
            self.lax_float()
        }
    }
    fn strict_float(&self) -> ValResult<f64>;
    fn lax_float(&self) -> ValResult<f64> {
        self.strict_float()
    }

    fn validate_dict<'data>(&'data self, strict: bool) -> ValResult<GenericMapping<'data>> {
        if strict {
            self.strict_dict()
        } else {
            self.lax_dict()
        }
    }
    fn strict_dict<'data>(&'data self) -> ValResult<GenericMapping<'data>>;
    fn lax_dict<'data>(&'data self) -> ValResult<GenericMapping<'data>> {
        self.strict_dict()
    }

    fn validate_typed_dict<'data>(
        &'data self,
        strict: bool,
        _from_attributes: bool,
    ) -> ValResult<GenericMapping<'data>> {
        self.validate_dict(strict)
    }

    fn validate_list<'data>(&'data self, strict: bool) -> ValResult<GenericSequence<'data>> {
        if strict {
            self.strict_list()
        } else {
            self.lax_list()
        }
    }
    fn strict_list<'data>(&'data self) -> ValResult<GenericSequence<'data>>;
    fn lax_list<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        self.strict_list()
    }

    fn validate_tuple<'data>(&'data self, strict: bool) -> ValResult<GenericSequence<'data>> {
        if strict {
            self.strict_tuple()
        } else {
            self.lax_tuple()
        }
    }
    fn strict_tuple<'data>(&'data self) -> ValResult<GenericSequence<'data>>;
    fn lax_tuple<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        self.strict_tuple()
    }

    fn validate_set<'data>(&'data self, strict: bool) -> ValResult<GenericSequence<'data>> {
        if strict {
            self.strict_set()
        } else {
            self.lax_set()
        }
    }
    fn strict_set<'data>(&'data self) -> ValResult<GenericSequence<'data>>;
    fn lax_set<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        self.strict_set()
    }

    fn validate_frozenset<'data>(&'data self, strict: bool) -> ValResult<GenericSequence<'data>> {
        if strict {
            self.strict_frozenset()
        } else {
            self.lax_frozenset()
        }
    }
    fn strict_frozenset<'data>(&'data self) -> ValResult<GenericSequence<'data>>;
    fn lax_frozenset<'data>(&'data self) -> ValResult<GenericSequence<'data>> {
        self.strict_frozenset()
    }

    fn validate_date(&self, strict: bool) -> ValResult<EitherDate> {
        if strict {
            self.strict_date()
        } else {
            self.lax_date()
        }
    }
    fn strict_date(&self) -> ValResult<EitherDate>;
    fn lax_date(&self) -> ValResult<EitherDate> {
        self.strict_date()
    }

    fn validate_time(&self, strict: bool) -> ValResult<EitherTime> {
        if strict {
            self.strict_time()
        } else {
            self.lax_time()
        }
    }
    fn strict_time(&self) -> ValResult<EitherTime>;
    fn lax_time(&self) -> ValResult<EitherTime> {
        self.strict_time()
    }

    fn validate_datetime(&self, strict: bool) -> ValResult<EitherDateTime> {
        if strict {
            self.strict_datetime()
        } else {
            self.lax_datetime()
        }
    }
    fn strict_datetime(&self) -> ValResult<EitherDateTime>;
    fn lax_datetime(&self) -> ValResult<EitherDateTime> {
        self.strict_datetime()
    }

    fn validate_timedelta(&self, strict: bool) -> ValResult<EitherTimedelta> {
        if strict {
            self.strict_timedelta()
        } else {
            self.lax_timedelta()
        }
    }
    fn strict_timedelta(&self) -> ValResult<EitherTimedelta>;
    fn lax_timedelta(&self) -> ValResult<EitherTimedelta> {
        self.strict_timedelta()
    }
}
