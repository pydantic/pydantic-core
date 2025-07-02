// This serializer is defined so that building a schema serializer containing an
// 'unset-sentinel' core schema doesn't crash. In practice, the serializer isn't
// used for model-like classes, as the 'fields' serializer takes care of omitting
// the fields from the output (the serializer can still be used if the 'unset-sentinel'
// core schema is used standalone (e.g. with a Pydantic type adapter), but this isn't
// something we explicitly support.

use std::borrow::Cow;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use serde::ser::Error;

use crate::common::unset_sentinel::get_unset_sentinel_object;
use crate::definitions::DefinitionsBuilder;
use crate::PydanticSerializationUnexpectedValue;

use super::{BuildSerializer, CombinedSerializer, Extra, TypeSerializer};

#[derive(Debug)]
pub struct UnsetSentinelSerializer {}

impl BuildSerializer for UnsetSentinelSerializer {
    const EXPECTED_TYPE: &'static str = "unset-sentinel";

    fn build(
        _schema: &Bound<'_, PyDict>,
        _config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        Ok(Self {}.into())
    }
}

impl_py_gc_traverse!(UnsetSentinelSerializer {});

impl TypeSerializer for UnsetSentinelSerializer {
    fn to_python(
        &self,
        value: &Bound<'_, PyAny>,
        _include: Option<&Bound<'_, PyAny>>,
        _exclude: Option<&Bound<'_, PyAny>>,
        _extra: &Extra,
    ) -> PyResult<PyObject> {
        let unset_obj = get_unset_sentinel_object(value.py());

        if value.is(unset_obj) {
            Ok(unset_obj.to_owned().into())
        } else {
            Err(
                PydanticSerializationUnexpectedValue::new_from_msg(Some("Expected 'UNSET' sentinel".to_string()))
                    .to_py_err(),
            )
        }
    }

    fn json_key<'a>(&self, key: &'a Bound<'_, PyAny>, extra: &Extra) -> PyResult<Cow<'a, str>> {
        self.invalid_as_json_key(key, extra, Self::EXPECTED_TYPE)
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        _value: &Bound<'_, PyAny>,
        _serializer: S,
        _include: Option<&Bound<'_, PyAny>>,
        _exclude: Option<&Bound<'_, PyAny>>,
        _extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        Err(Error::custom("'UNSET' can't be serialized to JSON".to_string()))
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}
