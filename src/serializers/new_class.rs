use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use crate::build_tools::SchemaDict;

use super::shared::{py_err_se_err, BuildSerializer, CombinedSerializer, Extra, TypeSerializer};

#[derive(Debug, Clone)]
pub struct NewClassSerializer {
    serializer: Box<CombinedSerializer>,
    dict_attr_name: Py<PyString>,
}

impl BuildSerializer for NewClassSerializer {
    const EXPECTED_TYPE: &'static str = "new-class";

    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let sub_schema: &PyDict = schema.get_as_req(intern!(py, "schema"))?;
        let serializer = Box::new(CombinedSerializer::build(sub_schema, config)?);
        let dict_attr_name = intern!(py, "__dict__").into_py(py);

        Ok(Self {
            serializer,
            dict_attr_name,
        }
        .into())
    }
}

impl NewClassSerializer {
    fn get_dict<'py>(&self, value: &'py PyAny) -> PyResult<&'py PyDict> {
        let attr = value.getattr(self.dict_attr_name.as_ref(value.py()))?;
        Ok(attr.cast_as::<PyDict>()?)
    }
}

impl TypeSerializer for NewClassSerializer {
    fn to_python(
        &self,
        value: &PyAny,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> PyResult<PyObject> {
        let dict = self.get_dict(value)?;
        self.serializer.to_python(dict, include, exclude, extra)
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &PyAny,
        serializer: S,
        include: Option<&PyAny>,
        exclude: Option<&PyAny>,
        extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        let dict = self.get_dict(value).map_err(py_err_se_err)?;
        self.serializer
            .serde_serialize(dict, serializer, include, exclude, extra)
    }
}
