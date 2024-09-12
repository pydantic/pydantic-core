use std::sync::OnceLock;

use pyo3::{
    intern,
    types::{PyAnyMethods, PyDict, PyDictMethods, PyTuple, PyTupleMethods, PyType},
    Bound, Py, PyAny, PyObject, PyResult, Python,
};

use crate::{
    definitions::DefinitionsBuilder,
    errors::{ErrorTypeDefaults, ValError, ValResult},
    input::Input,
    recursion_guard::RecursionGuard,
};

use super::{BuildValidator, CombinedValidator, SchemaValidator, ValidationState, Validator};

#[derive(Debug)]
pub struct NestedValidator {
    cls: Py<PyType>,
    name: String,
    get_validator: Py<PyAny>,
    validator: OnceLock<PyResult<Py<SchemaValidator>>>,
}

impl_py_gc_traverse!(NestedValidator {
    cls,
    get_validator,
    validator
});

impl BuildValidator for NestedValidator {
    const EXPECTED_TYPE: &'static str = "nested";

    fn build(
        schema: &Bound<'_, PyDict>,
        _config: Option<&Bound<'_, PyDict>>,
        _definitions: &mut DefinitionsBuilder<super::CombinedValidator>,
    ) -> PyResult<super::CombinedValidator> {
        let py = schema.py();

        let get_validator = schema.get_item(intern!(py, "get_info"))?.unwrap().unbind();

        let cls = schema
            .get_item(intern!(py, "cls"))?
            .unwrap()
            .downcast::<PyType>()?
            .clone();

        let name = cls.getattr(intern!(py, "__name__"))?.extract()?;

        Ok(CombinedValidator::Nested(NestedValidator {
            cls: cls.clone().unbind(),
            name,
            get_validator: get_validator,
            validator: OnceLock::new(),
        }))
    }
}

impl NestedValidator {
    fn nested_validator<'py>(&self, py: Python<'py>) -> PyResult<&Py<SchemaValidator>> {
        self.validator
            .get_or_init(|| {
                Ok(self
                    .get_validator
                    .bind(py)
                    .call((), None)?
                    .downcast::<PyTuple>()?
                    .get_item(1)?
                    .downcast::<SchemaValidator>()?
                    .clone()
                    .unbind())
            })
            .as_ref()
            .map_err(|e| e.clone_ref(py))
    }
}

impl Validator for NestedValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        let Some(id) = input.as_python().map(py_identity) else {
            return self
                .nested_validator(py)?
                .bind(py)
                .get()
                .validator
                .validate(py, input, state);
        };

        // Python objects can be cyclic, so need recursion guard
        let Ok(mut guard) = RecursionGuard::new(state, id, self.cls.as_ptr() as usize) else {
            return Err(ValError::new(ErrorTypeDefaults::RecursionLoop, input));
        };

        self.nested_validator(py)?
            .bind(py)
            .get()
            .validator
            .validate(py, input, guard.state())
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

fn py_identity(obj: &Bound<'_, PyAny>) -> usize {
    obj.as_ptr() as usize
}
