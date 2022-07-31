use std::fmt;

use pyo3::prelude::*;
use pyo3::types::{PyString, PyType};

use crate::errors::{InputValue, LocItem, ValResult};
use crate::input::datetime::EitherTime;

use super::datetime::{EitherDate, EitherDateTime, EitherTimedelta};
use super::return_enums::EitherBytes;
use super::{GenericArguments, GenericListLike, GenericMapping};

/// all types have three methods: `validate_*`, `strict_*`, `lax_*`
/// the convention is to either implement:
/// * `strict_*` & `lax_*` if they have different behavior
/// * or, `validate_*` and `strict_*` to just call `validate_*` if the behavior for strict and lax is the same
pub trait Input<'a>: fmt::Debug + ToPyObject {
    fn as_loc_item(&self, py: Python) -> LocItem;

    fn as_error_value(&'a self) -> InputValue<'a>;

    fn identity(&self) -> Option<usize> {
        None
    }

    fn is_none(&self) -> bool;

    fn is_type(&self, _class: &PyType) -> ValResult<bool> {
        Ok(false)
    }

    #[cfg_attr(has_no_coverage, no_coverage)]
    fn get_attr(&self, _name: &PyString) -> Option<&PyAny> {
        None
    }

    fn is_instance(&self, _class: &PyType) -> PyResult<bool> {
        Ok(false)
    }

    fn callable(&self) -> bool {
        false
    }

    fn validate_args(&'a self) -> ValResult<'a, GenericArguments<'a>>;

    fn validate_str(&'a self, py: Python<'a>, strict: bool) -> ValResult<&'a PyString> {
        if strict {
            self.strict_str(py)
        } else {
            self.lax_str(py)
        }
    }
    fn strict_str(&'a self, py: Python<'a>) -> ValResult<&'a PyString>;
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn lax_str(&'a self, py: Python<'a>) -> ValResult<&'a PyString> {
        self.strict_str(py)
    }

    fn validate_bytes(&'a self, py: Python, strict: bool) -> ValResult<EitherBytes<'a>> {
        if strict {
            self.strict_bytes(py)
        } else {
            self.lax_bytes(py)
        }
    }
    fn strict_bytes(&'a self, py: Python) -> ValResult<EitherBytes<'a>>;
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn lax_bytes(&'a self, py: Python) -> ValResult<EitherBytes<'a>> {
        self.strict_bytes(py)
    }

    fn validate_bool(&self, py: Python, strict: bool) -> ValResult<bool> {
        if strict {
            self.strict_bool()
        } else {
            self.lax_bool(py)
        }
    }
    fn strict_bool(&self) -> ValResult<bool>;
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn lax_bool(&self, _py: Python) -> ValResult<bool> {
        self.strict_bool()
    }

    fn validate_int(&self, py: Python, strict: bool) -> ValResult<i64> {
        if strict {
            self.strict_int()
        } else {
            self.lax_int(py)
        }
    }
    fn strict_int(&self) -> ValResult<i64>;
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn lax_int(&self, _py: Python) -> ValResult<i64> {
        self.strict_int()
    }

    fn validate_float(&self, py: Python, strict: bool) -> ValResult<f64> {
        if strict {
            self.strict_float()
        } else {
            self.lax_float(py)
        }
    }
    fn strict_float(&self) -> ValResult<f64>;
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn lax_float(&self, _py: Python) -> ValResult<f64> {
        self.strict_float()
    }

    fn validate_dict(&'a self, strict: bool) -> ValResult<GenericMapping<'a>> {
        if strict {
            self.strict_dict()
        } else {
            self.lax_dict()
        }
    }
    fn strict_dict(&'a self) -> ValResult<GenericMapping<'a>>;
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn lax_dict(&'a self) -> ValResult<GenericMapping<'a>> {
        self.strict_dict()
    }

    fn validate_typed_dict(&'a self, strict: bool, _from_attributes: bool) -> ValResult<GenericMapping<'a>> {
        self.validate_dict(strict)
    }

    fn validate_list(&'a self, strict: bool) -> ValResult<GenericListLike<'a>> {
        if strict {
            self.strict_list()
        } else {
            self.lax_list()
        }
    }
    fn strict_list(&'a self) -> ValResult<GenericListLike<'a>>;
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn lax_list(&'a self) -> ValResult<GenericListLike<'a>> {
        self.strict_list()
    }

    fn validate_tuple(&'a self, strict: bool) -> ValResult<GenericListLike<'a>> {
        if strict {
            self.strict_tuple()
        } else {
            self.lax_tuple()
        }
    }
    fn strict_tuple(&'a self) -> ValResult<GenericListLike<'a>>;
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn lax_tuple(&'a self) -> ValResult<GenericListLike<'a>> {
        self.strict_tuple()
    }

    fn validate_set(&'a self, strict: bool) -> ValResult<GenericListLike<'a>> {
        if strict {
            self.strict_set()
        } else {
            self.lax_set()
        }
    }
    fn strict_set(&'a self) -> ValResult<GenericListLike<'a>>;
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn lax_set(&'a self) -> ValResult<GenericListLike<'a>> {
        self.strict_set()
    }

    fn validate_frozenset(&'a self, strict: bool) -> ValResult<GenericListLike<'a>> {
        if strict {
            self.strict_frozenset()
        } else {
            self.lax_frozenset()
        }
    }
    fn strict_frozenset(&'a self) -> ValResult<GenericListLike<'a>>;
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn lax_frozenset(&'a self) -> ValResult<GenericListLike<'a>> {
        self.strict_frozenset()
    }

    fn validate_date(&self, py: Python, strict: bool) -> ValResult<EitherDate> {
        if strict {
            self.strict_date(py)
        } else {
            self.lax_date(py)
        }
    }
    fn strict_date(&self, py: Python) -> ValResult<EitherDate>;
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn lax_date(&self, py: Python) -> ValResult<EitherDate> {
        self.strict_date(py)
    }

    fn validate_time(&self, py: Python, strict: bool) -> ValResult<EitherTime> {
        if strict {
            self.strict_time(py)
        } else {
            self.lax_time(py)
        }
    }
    fn strict_time(&self, py: Python) -> ValResult<EitherTime>;
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn lax_time(&self, py: Python) -> ValResult<EitherTime> {
        self.strict_time(py)
    }

    fn validate_datetime(&self, py: Python, strict: bool) -> ValResult<EitherDateTime> {
        if strict {
            self.strict_datetime(py)
        } else {
            self.lax_datetime(py)
        }
    }
    fn strict_datetime(&self, py: Python) -> ValResult<EitherDateTime>;
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn lax_datetime(&self, py: Python) -> ValResult<EitherDateTime> {
        self.strict_datetime(py)
    }

    fn validate_timedelta(&self, py: Python, strict: bool) -> ValResult<EitherTimedelta> {
        if strict {
            self.strict_timedelta(py)
        } else {
            self.lax_timedelta(py)
        }
    }
    fn strict_timedelta(&self, py: Python) -> ValResult<EitherTimedelta>;
    #[cfg_attr(has_no_coverage, no_coverage)]
    fn lax_timedelta(&self, py: Python) -> ValResult<EitherTimedelta> {
        self.strict_timedelta(py)
    }
}
