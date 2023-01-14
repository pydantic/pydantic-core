use std::cell::RefCell;
use std::fmt::Debug;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::{intern, AsPyPointer};

use nohash_hasher::IntSet;
use serde::ser::Error;

use crate::build_tools::py_err;

use super::config::SerializationConfig;
use super::errors::PydanticSerializationUnexpectedValue;
use super::ob_type::ObTypeLookup;
use super::shared::CombinedSerializer;

/// Useful things which are passed around by type_serializers
#[derive(Clone)]
pub(crate) struct Extra<'a> {
    pub mode: &'a SerMode,
    pub slots: &'a [CombinedSerializer],
    pub ob_type_lookup: &'a ObTypeLookup,
    pub warnings: &'a CollectWarnings,
    pub by_alias: bool,
    pub exclude_unset: bool,
    pub exclude_defaults: bool,
    pub exclude_none: bool,
    pub round_trip: bool,
    pub config: &'a SerializationConfig,
    pub rec_guard: &'a SerRecursionGuard,
    // the next two are used for union logic
    pub error_on_fallback: bool,
    // "subclasses" could be generalised to some kind of "also allows similar values" in future if required
    pub allow_subclasses: bool,
}

impl<'a> Extra<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        py: Python<'a>,
        mode: &'a SerMode,
        slots: &'a [CombinedSerializer],
        by_alias: Option<bool>,
        warnings: &'a CollectWarnings,
        exclude_unset: Option<bool>,
        exclude_defaults: Option<bool>,
        exclude_none: Option<bool>,
        round_trip: Option<bool>,
        config: &'a SerializationConfig,
        rec_guard: &'a SerRecursionGuard,
    ) -> Self {
        Self {
            mode,
            slots,
            ob_type_lookup: ObTypeLookup::cached(py),
            warnings,
            by_alias: by_alias.unwrap_or(true),
            exclude_unset: exclude_unset.unwrap_or(false),
            exclude_defaults: exclude_defaults.unwrap_or(false),
            exclude_none: exclude_none.unwrap_or(false),
            round_trip: round_trip.unwrap_or(false),
            config,
            rec_guard,
            error_on_fallback: false,
            allow_subclasses: true,
        }
    }

    pub fn with_error_on_fallback(&self, allow_subclasses: bool) -> Self {
        let mut new_extra = self.clone();
        new_extra.error_on_fallback = true;
        new_extra.allow_subclasses = allow_subclasses;
        new_extra
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) struct ExtraOwned {
    mode: SerMode,
    slots: Vec<CombinedSerializer>,
    warnings: CollectWarnings,
    by_alias: bool,
    exclude_unset: bool,
    exclude_defaults: bool,
    exclude_none: bool,
    round_trip: bool,
    config: SerializationConfig,
    rec_guard: SerRecursionGuard,
    error_on_fallback: bool,
    allow_subclasses: bool,
}

impl ExtraOwned {
    pub fn new(extra: &Extra) -> Self {
        Self {
            mode: extra.mode.clone(),
            slots: extra.slots.to_vec(),
            warnings: extra.warnings.clone(),
            by_alias: extra.by_alias,
            exclude_unset: extra.exclude_unset,
            exclude_defaults: extra.exclude_defaults,
            exclude_none: extra.exclude_none,
            round_trip: extra.round_trip,
            config: extra.config.clone(),
            rec_guard: extra.rec_guard.clone(),
            error_on_fallback: extra.error_on_fallback,
            allow_subclasses: extra.allow_subclasses,
        }
    }

    pub fn to_extra<'py>(&'py self, py: Python<'py>) -> Extra<'py> {
        Extra {
            mode: &self.mode,
            slots: &self.slots,
            ob_type_lookup: ObTypeLookup::cached(py),
            warnings: &self.warnings,
            by_alias: self.by_alias,
            exclude_unset: self.exclude_unset,
            exclude_defaults: self.exclude_defaults,
            exclude_none: self.exclude_none,
            round_trip: self.round_trip,
            config: &self.config,
            rec_guard: &self.rec_guard,
            error_on_fallback: self.error_on_fallback,
            allow_subclasses: self.allow_subclasses,
        }
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) enum SerMode {
    Python,
    Json,
    Other(String),
}

impl From<Option<&str>> for SerMode {
    fn from(s: Option<&str>) -> Self {
        match s {
            Some("json") => SerMode::Json,
            Some("python") => SerMode::Python,
            Some(other) => SerMode::Other(other.to_string()),
            None => SerMode::Python,
        }
    }
}

impl ToPyObject for SerMode {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            SerMode::Python => intern!(py, "python").to_object(py),
            SerMode::Json => intern!(py, "json").to_object(py),
            SerMode::Other(s) => s.to_object(py),
        }
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) struct CollectWarnings {
    active: bool,
    warnings: RefCell<Option<Vec<String>>>,
}

impl CollectWarnings {
    pub(crate) fn new(active: Option<bool>) -> Self {
        Self {
            active: active.unwrap_or(true),
            warnings: RefCell::new(None),
        }
    }

    pub(crate) fn on_fallback_py(&self, field_type: &str, value: &PyAny, error_on_fallback: bool) -> PyResult<()> {
        if error_on_fallback {
            Err(PydanticSerializationUnexpectedValue::new_err(None))
        } else {
            self.fallback_warning(field_type, value);
            Ok(())
        }
    }

    pub(crate) fn on_fallback_ser<S: serde::ser::Serializer>(
        &self,
        field_type: &str,
        value: &PyAny,
        error_on_fallback: bool,
    ) -> Result<(), S::Error> {
        if error_on_fallback {
            Err(S::Error::custom("Unexpected value"))
        } else {
            self.fallback_warning(field_type, value);
            Ok(())
        }
    }

    fn fallback_warning(&self, field_type: &str, value: &PyAny) {
        if self.active {
            let type_name = value.get_type().name().unwrap_or("<unknown python object>");
            self.add_warning(format!(
                "Expected `{field_type}` but got `{type_name}` - serialized value may not be as expected"
            ));
        }
    }

    fn add_warning(&self, message: String) {
        let mut op_warnings = self.warnings.borrow_mut();
        if let Some(ref mut warnings) = *op_warnings {
            warnings.push(message);
        } else {
            *op_warnings = Some(vec![message]);
        }
    }

    pub(crate) fn final_check(&self, py: Python) -> PyResult<()> {
        if self.active {
            match *self.warnings.borrow() {
                Some(ref warnings) => {
                    let warnings = warnings.iter().map(|w| w.as_str()).collect::<Vec<_>>();
                    let message = format!("Pydantic serializer warnings:\n  {}", warnings.join("\n  "));
                    let user_warning_type = py.import("builtins")?.getattr("UserWarning")?;
                    PyErr::warn(py, user_warning_type, &message, 0)
                }
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }
}

/// we have `RecursionInfo` then a `RefCell` since `SerializeInfer.serialize` can't take a `&mut self`
#[derive(Default, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct RecursionInfo {
    ids: IntSet<usize>,
    /// as with `src/recursion_guard.rs` this is used as a backup in case the identity check recursion guard fails
    /// see #143
    depth: u16,
}

#[derive(Default, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct SerRecursionGuard {
    info: RefCell<RecursionInfo>,
}

impl SerRecursionGuard {
    const MAX_DEPTH: u16 = 200;

    pub fn add(&self, value: &PyAny) -> PyResult<usize> {
        // https://doc.rust-lang.org/std/collections/struct.HashSet.html#method.insert
        // "If the set did not have this value present, `true` is returned."
        let id = value.as_ptr() as usize;
        let mut info = self.info.borrow_mut();
        if !info.ids.insert(id) {
            py_err!(PyValueError; "Circular reference detected (id repeated)")
        } else if info.depth > Self::MAX_DEPTH {
            py_err!(PyValueError; "Circular reference detected (depth exceeded)")
        } else {
            info.depth += 1;
            Ok(id)
        }
    }

    pub fn pop(&self, id: usize) {
        let mut info = self.info.borrow_mut();
        info.depth -= 1;
        info.ids.remove(&id);
    }
}
