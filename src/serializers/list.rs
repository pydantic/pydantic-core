use crate::build_tools::SchemaDict;
use crate::serializers::SerializeCombinedSerializer;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use serde::ser::SerializeSeq;

use super::{build_serializer, py_err_to_serde, BuildSerializer, CombinedSerializer, Serializer};

#[derive(Debug, Clone)]
pub struct ListSerializer {
    item_serializer: Box<CombinedSerializer>,
}

impl BuildSerializer for ListSerializer {
    const EXPECTED_TYPE: &'static str = "list";

    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        let item_serializer: &PyDict = schema.get_as_req(pyo3::intern!(schema.py(), "item_serializer"))?;
        let item_serializer = Box::new(build_serializer(item_serializer, config)?);
        Ok(Self { item_serializer }.into())
    }
}

impl Serializer for ListSerializer {
    fn to_python(&self, py: Python, value: &PyAny, format: Option<&str>) -> PyResult<PyObject> {
        let list: &PyList = value.cast_as()?;
        let items = list
            .iter()
            .map(|item| self.item_serializer.to_python(py, item, format))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(items.into_py(py))
    }

    fn serde_serialize<S>(&self, value: &PyAny, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let list: &PyList = value.cast_as().map_err(py_err_to_serde)?;

        let mut seq = serializer.serialize_seq(Some(list.len()))?;
        for value in list.iter() {
            let scs = SerializeCombinedSerializer {
                value,
                serializer: &self.item_serializer,
            };
            seq.serialize_element(&scs)?;
        }
        seq.end()
    }
}
