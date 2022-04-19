use pyo3::intern;
use pyo3::once_cell::GILOnceCell;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

use super::{build_validator, Extra, ValResult, Validator};
use crate::build_macros::{dict_get, dict_get_required, py_error};
use crate::errors::as_internal;
use crate::input::Input;

#[derive(Debug, Clone)]
pub struct ModelClassValidator {
    validator: Box<dyn Validator>,
    class: PyObject,
    new_method: PyObject,
}

impl ModelClassValidator {
    pub const EXPECTED_TYPE: &'static str = "model-class";
}

impl Validator for ModelClassValidator {
    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<Box<dyn Validator>> {
        let class = dict_get_required!(schema, "class", &PyAny)?;
        let new_method = class.getattr("__new__")?;
        if !new_method.is_callable() {
            return py_error!("Got class '{:?}' with uncallable __new__ method", class);
        }
        let model_schema = dict_get_required!(schema, "model", &PyDict)?;
        let model_type = dict_get!(model_schema, "type", String);
        if model_type != Some("model".to_string()) {
            return py_error!("model-class expected a 'model' schema, got {:?}", model_type);
        }
        Ok(Box::new(Self {
            validator: build_validator(model_schema, config)?,
            class: class.into(),
            new_method: new_method.into(),
        }))
    }

    fn validate(&self, py: Python, input: &dyn Input, extra: &Extra) -> ValResult<PyObject> {
        let output = self.validator.validate(py, input, extra)?;
        self.create_class(py, output).map_err(as_internal)
    }

    fn clone_dyn(&self) -> Box<dyn Validator> {
        Box::new(self.clone())
    }
}

impl ModelClassValidator {
    /// utility used to avoid lots of `.map_err(as_internal)` in `validate()`
    #[inline]
    fn create_class(&self, py: Python, output: PyObject) -> PyResult<PyObject> {
        let t: &PyTuple = output.extract(py)?;
        let model_dict = t.get_item(0)?;
        let fields_set = t.get_item(1)?;

        let instance = self.new_method.call(py, (self.class.as_ref(py),), None)?;
        let setattr = get_setattr(py);
        setattr.call((&instance, intern!(py, "__dict__"), model_dict), None)?;
        setattr.call((&instance, intern!(py, "__fields_set__"), fields_set), None)?;

        Ok(instance)
    }
}

static SET_ATTR_CELL: GILOnceCell<PyObject> = GILOnceCell::new();

pub fn get_setattr(py: Python<'_>) -> &PyAny {
    SET_ATTR_CELL
        .get_or_init(py, || py.eval("object.__setattr__", None, None).unwrap().into())
        .as_ref(py)
}
