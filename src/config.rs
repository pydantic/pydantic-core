use std::convert::Infallible;

use pyo3::{
    intern,
    types::{PyAnyMethods, PyString},
    Borrowed, Bound, FromPyObject, IntoPyObject, PyAny, PyResult, Python,
};

use crate::{
    build_tools::{py_schema_error_type, ExtraBehavior},
    validators::{Revalidate, TemporalUnitMode, ValBytesMode},
};

#[derive(FromPyObject, IntoPyObject, Default, Clone, Debug)]
pub struct CoreConfig {
    pub loc_by_alias: Option<bool>,
    pub extra_fields_behavior: Option<ExtraBehavior>,
    pub validate_by_alias: Option<bool>,
    pub validate_by_name: Option<bool>,
    pub val_json_bytes: Option<ValBytesMode>,
    pub val_temporal_unit: Option<TemporalUnitMode>,
    pub revalidate_instances: Option<Revalidate>,
    pub microseconds_precision: Option<MicrosecondsPrecisionOverflowBehavior>,
    pub strict: Option<bool>,
    pub allow_inf_nan: Option<bool>,
}

/// Wrapper around the speedate config value to add Python conversions
#[derive(Debug, Clone, Copy, PartialEq)]
struct MicrosecondsPrecisionOverflowBehavior(pub speedate::MicrosecondsPrecisionOverflowBehavior);

impl FromPyObject<'_> for MicrosecondsPrecisionOverflowBehavior {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        let s: &str = ob.extract()?;
        s.parse().map(MicrosecondsPrecisionOverflowBehavior).map_err(|_| {
            py_schema_error_type!("Invalid `microseconds_precision`, must be one of \"truncate\" or \"error\"")
        })
    }
}

impl<'py> IntoPyObject<'py> for MicrosecondsPrecisionOverflowBehavior {
    type Target = PyString;

    type Output = Borrowed<'py, 'py, PyString>;

    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let s = match self.0 {
            speedate::MicrosecondsPrecisionOverflowBehavior::Truncate => intern!(py, "truncate"),
            speedate::MicrosecondsPrecisionOverflowBehavior::Error => intern!(py, "error"),
        };
        Ok(s.as_borrowed())
    }
}
