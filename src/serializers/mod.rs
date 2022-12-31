use std::fmt::Debug;

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

use crate::build_context::BuildContext;
use crate::SchemaValidator;

use extra::{Extra, SerMode};
pub use shared::CombinedSerializer;
use shared::{to_json_bytes, BuildSerializer, TypeSerializer};
use type_serializers::timedelta::TimedeltaMode;

mod extra;
mod include_exclude;
mod ob_type;
mod shared;
mod type_serializers;

#[pyclass(module = "pydantic_core._pydantic_core")]
#[derive(Debug, Clone)]
pub struct SchemaSerializer {
    serializer: CombinedSerializer,
    slots: Vec<CombinedSerializer>,
    json_size: usize,
    timedelta_mode: TimedeltaMode,
}

#[pymethods]
impl SchemaSerializer {
    #[new]
    pub fn py_new(py: Python, schema: &PyDict, config: Option<&PyDict>) -> PyResult<Self> {
        let schema = SchemaValidator::validate_schema(py, schema)?;
        let mut build_context = BuildContext::for_schema(schema)?;
        let serializer = CombinedSerializer::build(schema.cast_as()?, config, &mut build_context)?;
        Ok(Self {
            serializer,
            slots: build_context.into_slots_ser()?,
            json_size: 1024,
            timedelta_mode: TimedeltaMode::from_config(config)?,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn to_python(
        &self,
        py: Python,
        value: &PyAny,
        mode: Option<&str>,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        by_alias: Option<bool>,
        exclude_unset: Option<bool>,
        exclude_defaults: Option<bool>,
        exclude_none: Option<bool>,
        round_trip: Option<bool>,
    ) -> PyResult<PyObject> {
        let mode: SerMode = mode.into();
        let extra = Extra::new(
            py,
            &mode,
            &self.slots,
            by_alias,
            exclude_unset,
            exclude_defaults,
            exclude_none,
            round_trip,
            self.timedelta_mode,
        );
        let v = self.serializer.to_python(value, include, exclude, &extra)?;
        extra.warnings.final_check(py)?;
        Ok(v)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn to_json(
        &mut self,
        py: Python,
        value: &PyAny,
        indent: Option<usize>,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        by_alias: Option<bool>,
        exclude_unset: Option<bool>,
        exclude_defaults: Option<bool>,
        exclude_none: Option<bool>,
        round_trip: Option<bool>,
    ) -> PyResult<PyObject> {
        let mode = SerMode::Json;
        let extra = Extra::new(
            py,
            &mode,
            &self.slots,
            by_alias,
            exclude_unset,
            exclude_defaults,
            exclude_none,
            round_trip,
            self.timedelta_mode,
        );
        let bytes = to_json_bytes(
            value,
            &self.serializer,
            include,
            exclude,
            &extra,
            indent,
            self.json_size,
        )?;

        extra.warnings.final_check(py)?;

        self.json_size = bytes.len();
        let py_bytes = PyBytes::new(py, &bytes);
        Ok(py_bytes.into())
    }

    pub fn __repr__(&self) -> String {
        format!(
            "SchemaSerializer(serializer={:#?}, slots={:#?})",
            self.serializer, self.slots
        )
    }
}
