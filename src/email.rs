use pyo3::once_cell::GILOnceCell;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::vendored::email_address::EmailAddress;
use crate::SchemaValidator;

static SCHEMA_DEFINITION_EMAIL: GILOnceCell<SchemaValidator> = GILOnceCell::new();

#[pyclass(name = "Email", module = "pydantic_core._pydantic_core")]
#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct PyEmail {
    lib_email: EmailAddress,
}

impl PyEmail {
    pub fn new(lib_email: EmailAddress) -> Self {
        Self { lib_email }
    }

    pub fn into_email(self) -> EmailAddress {
        self.lib_email
    }
}

fn build_schema_validator(py: Python, schema_type: &str) -> SchemaValidator {
    let schema: &PyDict = PyDict::new(py);
    schema.set_item("type", schema_type).unwrap();
    SchemaValidator::py_new(py, schema, None).unwrap()
}

#[pymethods]
impl PyEmail {
    #[new]
    pub fn py_new(py: Python, email: &PyAny) -> PyResult<Self> {
        let schema_obj = SCHEMA_DEFINITION_EMAIL
            .get_or_init(py, || build_schema_validator(py, "email"))
            .validate_python(py, email, None, None)?;
        schema_obj.extract(py)
    }

    #[getter]
    pub fn name(&self) -> &str {
        self.lib_email.display_part()
    }

    #[getter]
    pub fn email(&self) -> String {
        self.lib_email.email()
    }

    #[getter]
    pub fn domain(&self) -> &str {
        self.lib_email.domain()
    }

    #[getter]
    pub fn local_part(&self) -> &str {
        self.lib_email.local_part()
    }

    #[getter]
    pub fn original_email(&self) -> &str {
        self.lib_email.as_str()
    }

    pub fn __str__(&self) -> &str {
        self.lib_email.as_str()
    }

    pub fn __repr__(&self) -> String {
        format!("Email('{}')", self.lib_email.as_str())
    }
}
