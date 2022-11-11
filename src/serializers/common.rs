use std::str::FromStr;

use crate::build_tools::py_error_type;
use crate::serializers::any::common_serialize;
use pyo3::prelude::*;

use super::any::{ObType, ObTypeLookup};
use super::{CombinedSerializer, TypeSerializer};

#[derive(Debug, Clone)]
pub struct CommonSerializer {
    ob_type: ObType,
}

impl CommonSerializer {
    pub fn build(type_: &str) -> PyResult<CombinedSerializer> {
        let ob_type = ObType::from_str(type_).map_err(|_| py_error_type!("Invalid type: {}", type_))?;
        Ok(Self { ob_type }.into())
    }
}

impl TypeSerializer for CommonSerializer {
    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        ob_type_lookup: &ObTypeLookup,
    ) -> Result<S::Ok, S::Error> {
        common_serialize(value, &self.ob_type, serializer, ob_type_lookup)
    }
}
