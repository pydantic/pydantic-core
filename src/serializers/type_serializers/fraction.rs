use std::borrow::Cow;
use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::LazyLock;
use crate::definitions::DefinitionsBuilder;
use crate::serializers::infer::{infer_json_key_known, infer_serialize_known, infer_to_python_known};
use crate::serializers::ob_type::{IsType, ObType};

use super::{
    infer_json_key, infer_serialize, infer_to_python, BuildSerializer, CombinedSerializer, Extra, TypeSerializer,
};

#[derive(Debug)]
pub struct FractionSerializer {}

static FRACTION_SERIALIZER: LazyLock<Arc<CombinedSerializer>> = LazyLock::new(|| Arc::new(FractionSerializer {}.into()));

impl BuildSerializer for FractionSerializer {
    const EXPECTED_TYPE: &'static str = "decimal";

    fn build(
        _schema: &Bound<'_, PyDict>,
        _config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<Arc<CombinedSerializer>>,
    ) -> PyResult<Arc<CombinedSerializer>> {
        Ok(FRACTION_SERIALIZER.clone())
    }
}

impl_py_gc_traverse!(FractionSerializer {});

impl TypeSerializer for FractionSerializer {
    fn to_python(
        &self,
        value: &Bound<'_, PyAny>,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<Py<PyAny>> {
        let _py = value.py();
        println!("[RUST] FractionSerializer to_python called");
        match extra.ob_type_lookup.is_type(value, ObType::Fraction) {
            IsType::Exact | IsType::Subclass => infer_to_python_known(ObType::Fraction, value, include, exclude, extra),
            IsType::False => {
                extra.warnings.on_fallback_py(self.get_name(), value, extra)?;
                infer_to_python(value, include, exclude, extra)
            }
        }
    }

    fn json_key<'a>(&self, key: &'a Bound<'_, PyAny>, extra: &Extra) -> PyResult<Cow<'a, str>> {
        match extra.ob_type_lookup.is_type(key, ObType::Fraction) {
            IsType::Exact | IsType::Subclass => infer_json_key_known(ObType::Fraction, key, extra),
            IsType::False => {
                extra.warnings.on_fallback_py(self.get_name(), key, extra)?;
                infer_json_key(key, extra)
            }
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Bound<'_, PyAny>,
        serializer: S,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        match extra.ob_type_lookup.is_type(value, ObType::Fraction) {
            IsType::Exact | IsType::Subclass => {
                infer_serialize_known(ObType::Fraction, value, serializer, include, exclude, extra)
            }
            IsType::False => {
                extra.warnings.on_fallback_ser::<S>(self.get_name(), value, extra)?;
                infer_serialize(value, serializer, include, exclude, extra)
            }
        }
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}
