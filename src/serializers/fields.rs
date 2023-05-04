use std::borrow::Cow;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use super::extra::Extra;
use super::shared::{CombinedSerializer, TypeSerializer};

/// representation of a field for serialization, used by `TypedDictSerializer` and `ModelFieldsSerializer`,
/// and maybe more
#[derive(Debug, Clone)]
pub(super) struct FieldSerializer {
    pub key_py: Py<PyString>,
    pub alias: Option<String>,
    pub alias_py: Option<Py<PyString>>,
    // None serializer means exclude
    pub serializer: Option<CombinedSerializer>,
    pub required: bool,
}

impl FieldSerializer {
    pub fn new(
        py: Python,
        key_py: Py<PyString>,
        alias: Option<String>,
        serializer: Option<CombinedSerializer>,
        required: bool,
    ) -> Self {
        let alias_py = alias.as_ref().map(|alias| PyString::new(py, alias.as_str()).into());
        Self {
            key_py,
            alias,
            alias_py,
            serializer,
            required,
        }
    }

    pub fn get_key_py<'py>(&'py self, py: Python<'py>, extra: &Extra) -> &'py PyAny {
        if extra.by_alias {
            if let Some(ref alias_py) = self.alias_py {
                return alias_py.as_ref(py);
            }
        }
        self.key_py.as_ref(py)
    }

    pub fn get_key_json<'a>(&'a self, key_str: &'a str, extra: &Extra) -> Cow<'a, str> {
        if extra.by_alias {
            if let Some(ref alias) = self.alias {
                return Cow::Borrowed(alias.as_str());
            }
        }
        Cow::Borrowed(key_str)
    }

    pub fn to_python(
        &self,
        output_dict: &PyDict,
        value: &PyAny,
        next_include: Option<&PyAny>,
        next_exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<()> {
        if let Some(ref serializer) = self.serializer {
            if !exclude_default(value, extra, serializer)? {
                let value = serializer.to_python(value, next_include, next_exclude, extra)?;
                let output_key = self.get_key_py(output_dict.py(), extra);
                output_dict.set_item(output_key, value)?;
            }
        }
        Ok(())
    }
}

pub(super) fn exclude_default(value: &PyAny, extra: &Extra, serializer: &CombinedSerializer) -> PyResult<bool> {
    if extra.exclude_defaults {
        if let Some(default) = serializer.get_default(value.py())? {
            if value.eq(default)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
