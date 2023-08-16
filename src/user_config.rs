use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use crate::tools::SchemaDict;

pub struct UserConfig<'a> {
    // The config for the current target (if any)
    pub target: Option<&'a PyDict>,
    // The global, top-level flags set (if any)
    pub flags: Option<&'a PyDict>,
}

impl<'a> UserConfig<'a> {
    pub fn new<'b>(target: Option<&'b PyDict>, flags: Option<&'b PyDict>) -> UserConfig<'b> {
        UserConfig { target, flags }
    }

    pub fn to_owned(&self, py: Python) -> OwnedUserConfig {
        OwnedUserConfig::from_reffed(py, self)
    }

    /// Empty config, as if user specified nothing,
    /// some usages needing the config object don't have real config available so can use this
    pub fn default() -> UserConfig<'a> {
        UserConfig {
            target: None,
            flags: None,
        }
    }

    /// A new holder with the different target config, flags unchanged.
    pub fn with_new_target<'b>(&'b self, target: Option<&'b PyDict>) -> UserConfig<'b> {
        UserConfig {
            target,
            flags: self.flags,
        }
    }

    /// Extracts a config value matching the specified type.
    /// First checks target config, if doesn't exist will return from flags
    /// If neither exist, will return None
    pub fn get_conf<T>(&'a self, py_key: &PyString) -> Option<T>
    where
        T: FromPyObject<'a>,
    {
        // First try from target config:
        if let Some(t) = self.target.get_as(py_key).unwrap_or(None) {
            return Some(t);
        }

        // Then try from flags:
        if let Some(t) = self.flags.get_as(py_key).unwrap_or(None) {
            return Some(t);
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct OwnedUserConfig {
    // Either None or PyDicts:
    pub target: PyObject,
    pub flags: PyObject,
}

impl OwnedUserConfig {
    // Converts back to the reffed holder which has all the methods
    pub fn to_reffed<'a>(&'a self, py: Python<'a>) -> UserConfig<'a> {
        UserConfig {
            target: if self.target.is_none(py) {
                None
            } else {
                unsafe { Some(self.target.downcast_unchecked(py)) }
            },
            flags: if self.flags.is_none(py) {
                None
            } else {
                unsafe { Some(self.flags.downcast_unchecked(py)) }
            },
        }
    }

    fn from_reffed(py: Python, reffed: &UserConfig) -> Self {
        Self {
            target: match reffed.target {
                Some(conf) => conf.to_object(py),
                None => py.None(),
            },
            flags: match reffed.flags {
                Some(conf) => conf.to_object(py),
                None => py.None(),
            },
        }
    }
}

impl_py_gc_traverse!(OwnedUserConfig { target, flags });
