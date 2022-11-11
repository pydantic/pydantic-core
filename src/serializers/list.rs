use crate::build_tools::SchemaDict;
use pyo3::prelude::*;
use pyo3::types::PyList;
use serde::ser::SerializeSeq;

macro_rules! build_sequence_serializer {
    ($struct_name:ident, $expected_type:literal, $type_:ty, $any_function_name:ident) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name {
            item_serializer: Option<Box<super::CombinedSerializer>>,
        }

        impl super::BuildSerializer for $struct_name {
            const EXPECTED_TYPE: &'static str = $expected_type;

            fn build(
                schema: &pyo3::types::PyDict,
                config: Option<&pyo3::types::PyDict>,
            ) -> PyResult<super::CombinedSerializer> {
                let item_serializer =
                    match schema.get_as::<&pyo3::types::PyDict>(pyo3::intern!(schema.py(), "items_schema"))? {
                        Some(item_serializer) => Some(Box::new(super::build_serializer(item_serializer, config)?)),
                        None => None,
                    };
                Ok(Self { item_serializer }.into())
            }
        }

        pub fn $any_function_name<S: serde::ser::Serializer>(
            value: &PyAny,
            serializer: S,
            ob_type_lookup: &super::any::ObTypeLookup,
        ) -> Result<S::Ok, S::Error> {
            let py_seq: $type_ = value.cast_as().map_err(super::py_err_se_err)?;

            let mut seq = serializer.serialize_seq(Some(py_seq.len()))?;
            for element in py_seq {
                seq.serialize_element(&super::any::SerializeInfer::new(element, ob_type_lookup))?
            }
            seq.end()
        }
    };
}
pub(crate) use build_sequence_serializer;

build_sequence_serializer!(ListSerializer, "list", &PyList, serialize_list_any);

impl super::TypeSerializer for ListSerializer {
    fn to_python(&self, py: Python, value: &PyAny, format: Option<&str>) -> PyResult<PyObject> {
        match self.item_serializer {
            Some(ref item_serializer) => {
                let py_seq: &PyList = value.cast_as()?;
                let items = py_seq
                    .iter()
                    .map(|item| item_serializer.to_python(py, item, format))
                    .collect::<PyResult<Vec<_>>>()?;
                Ok(items.into_py(py))
            }
            None => Ok(value.into_py(py)),
        }
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        ob_type_lookup: &super::any::ObTypeLookup,
    ) -> Result<S::Ok, S::Error> {
        match self.item_serializer {
            Some(ref item_serializer) => {
                let py_seq: &PyList = value.cast_as().map_err(super::py_err_se_err)?;

                let mut seq = serializer.serialize_seq(Some(py_seq.len()))?;
                for value in py_seq.iter() {
                    let item_serialize = super::PydanticSerializer::new(value, item_serializer, ob_type_lookup);
                    seq.serialize_element(&item_serialize)?;
                }
                seq.end()
            }
            None => serialize_list_any(value, serializer, ob_type_lookup),
        }
    }
}
