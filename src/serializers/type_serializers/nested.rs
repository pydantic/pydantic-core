use std::{borrow::Cow, sync::OnceLock};

use pyo3::{
    intern,
    types::{PyAnyMethods, PyDict, PyDictMethods, PyTuple, PyType},
    Bound, Py, PyAny, PyObject, PyResult, Python,
};

use crate::{
    definitions::DefinitionsBuilder,
    serializers::{
        shared::{BuildSerializer, TypeSerializer},
        CombinedSerializer, Extra,
    },
    SchemaSerializer,
};

#[derive(Debug)]
pub struct NestedSerializer {
    model: Py<PyType>,
    name: String,
    get_serializer: Py<PyAny>,
    serializer: OnceLock<PyResult<Py<SchemaSerializer>>>,
}

impl_py_gc_traverse!(NestedSerializer {
    model,
    get_serializer,
    serializer
});

impl BuildSerializer for NestedSerializer {
    const EXPECTED_TYPE: &'static str = "nested";

    fn build(
        schema: &Bound<'_, PyDict>,
        _config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();

        let get_serializer = schema
            .get_item(intern!(py, "get_info"))?
            .expect("Invalid core schema for `nested` type, no `get_info`")
            .unbind();

        let model = schema
            .get_item(intern!(py, "cls"))?
            .expect("Invalid core schema for `nested` type, no `model`")
            .downcast::<PyType>()
            .expect("Invalid core schema for `nested` type, not a `PyType`")
            .clone();

        let name = model.getattr(intern!(py, "__name__"))?.extract()?;

        Ok(CombinedSerializer::Nested(NestedSerializer {
            model: model.clone().unbind(),
            name,
            get_serializer,
            serializer: OnceLock::new(),
        }))
    }
}

impl NestedSerializer {
    fn nested_serializer<'py>(&self, py: Python<'py>) -> PyResult<&Py<SchemaSerializer>> {
        self.serializer
            .get_or_init(|| {
                Ok(self
                    .get_serializer
                    .bind(py)
                    .call((), None)?
                    .downcast::<PyTuple>()?
                    .get_item(2)?
                    .downcast::<SchemaSerializer>()?
                    .clone()
                    .unbind())
            })
            .as_ref()
            .map_err(|e| e.clone_ref(py))
    }
}

impl TypeSerializer for NestedSerializer {
    fn to_python(
        &self,
        value: &Bound<'_, PyAny>,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        mut extra: &Extra,
    ) -> PyResult<PyObject> {
        let mut guard = extra.recursion_guard(value, self.model.as_ptr() as usize)?;

        self.nested_serializer(value.py())?
            .bind(value.py())
            .get()
            .serializer
            .to_python(value, include, exclude, guard.state())
    }

    fn json_key<'a>(&self, key: &'a Bound<'_, PyAny>, extra: &Extra) -> PyResult<Cow<'a, str>> {
        self.nested_serializer(key.py())?
            .bind(key.py())
            .get()
            .serializer
            .json_key(key, extra)
    }

    fn serde_serialize<S: serde::ser::Serializer>(
        &self,
        value: &Bound<'_, PyAny>,
        serializer: S,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        mut extra: &Extra,
    ) -> Result<S::Ok, S::Error> {
        use super::py_err_se_err;

        let mut guard = extra
            .recursion_guard(value, self.model.as_ptr() as usize)
            .map_err(py_err_se_err)?;

        self.nested_serializer(value.py())
            // FIXME(BoxyUwU): Don't unwrap this
            .unwrap()
            .bind(value.py())
            .get()
            .serializer
            .serde_serialize(value, serializer, include, exclude, guard.state())
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}
