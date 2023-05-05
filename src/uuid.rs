use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use pyo3::once_cell::GILOnceCell;
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;
use pyo3::types::PyDict;
use uuid::Uuid;

use crate::SchemaValidator;

static SCHEMA_DEFINITION_UUID: GILOnceCell<SchemaValidator> = GILOnceCell::new();

#[pyclass(name = "Uuid", module = "pydantic_core._pydantic_core")]
#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct PyUuid {
    lib_uuid: Uuid,
}

impl PyUuid {
    pub fn new(lib_uuid: Uuid) -> Self {
        Self { lib_uuid }
    }

    pub fn into_uuid(self) -> Uuid {
        self.lib_uuid
    }
}

fn build_schema_validator(py: Python, schema_type: &str) -> SchemaValidator {
    let schema: &PyDict = PyDict::new(py);
    schema.set_item("type", schema_type).unwrap();
    SchemaValidator::py_new(py, schema, None).unwrap()
}

#[pymethods]
impl PyUuid {
    #[new]
    pub fn py_new(py: Python, uuid: &PyAny) -> PyResult<Self> {
        let schema_obj = SCHEMA_DEFINITION_UUID
            .get_or_init(py, || build_schema_validator(py, "uuid"))
            .validate_python(py, uuid, None, None, None)?;
        schema_obj.extract(py)
    }

    #[getter]
    pub fn urn(&self) -> String {
        format!("{}", self.lib_uuid.urn())
    }

    #[getter]
    pub fn variant(&self) -> String {
        format!("{}", self.lib_uuid.get_variant())
    }

    #[getter]
    pub fn version(&self) -> usize {
        self.lib_uuid.get_version_num()
    }

    pub fn __str__(&self) -> String {
        format!("{}", self.lib_uuid.hyphenated())
    }

    pub fn __repr__(&self) -> String {
        format!("Uuid('{}')", self.lib_uuid)
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Lt => Ok(self.lib_uuid < other.lib_uuid),
            CompareOp::Le => Ok(self.lib_uuid <= other.lib_uuid),
            CompareOp::Eq => Ok(self.lib_uuid == other.lib_uuid),
            CompareOp::Ne => Ok(self.lib_uuid != other.lib_uuid),
            CompareOp::Gt => Ok(self.lib_uuid > other.lib_uuid),
            CompareOp::Ge => Ok(self.lib_uuid >= other.lib_uuid),
        }
    }

    fn __hash__(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.lib_uuid.to_string().hash(&mut s);
        s.finish()
    }

    fn __bool__(&self) -> bool {
        true // an empty string is not a valid UUID
    }

    pub fn __deepcopy__(&self, py: Python, _memo: &PyDict) -> Py<PyAny> {
        self.clone().into_py(py)
    }
}
