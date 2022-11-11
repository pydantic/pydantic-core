use crate::build_tools::SchemaDict;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};
use serde::ser::SerializeSeq;

use super::list::build_sequence_serializer;

build_sequence_serializer!(TupleSerializer, "tuple", &PyTuple, serialize_tuple_any);

impl super::TypeSerializer for TupleSerializer {
    fn to_python(&self, py: Python, value: &PyAny, format: Option<&str>) -> PyResult<PyObject> {
        match self.item_serializer {
            Some(ref item_serializer) => {
                let py_seq: &PyTuple = value.cast_as()?;
                let items = py_seq
                    .iter()
                    .map(|item| item_serializer.to_python(py, item, format))
                    .collect::<PyResult<Vec<_>>>()?;
                Ok(items.into_py(py))
            }
            None => Ok(value.into_py(py)),
        }
    }

    fn to_python_json(
        &self,
        py: Python,
        value: &PyAny, // TODO "exclude" arguments
    ) -> PyResult<PyObject> {
        let py_tuple: &PyTuple = value.cast_as()?;
        match self.item_serializer {
            Some(ref item_serializer) => {
                let items = py_tuple
                    .iter()
                    .map(|item| item_serializer.to_python_json(py, item))
                    .collect::<PyResult<Vec<_>>>()?;
                Ok(items.into_py(py))
            }
            None => Ok(PyList::new(py, py_tuple).into_py(py)),
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
                let py_seq: &PyTuple = value.cast_as().map_err(super::py_err_se_err)?;

                let mut seq = serializer.serialize_seq(Some(py_seq.len()))?;
                for value in py_seq.iter() {
                    let item_serialize = super::PydanticSerializer::new(value, item_serializer, ob_type_lookup);
                    seq.serialize_element(&item_serialize)?;
                }
                seq.end()
            }
            None => serialize_tuple_any(value, serializer, ob_type_lookup),
        }
    }
}
