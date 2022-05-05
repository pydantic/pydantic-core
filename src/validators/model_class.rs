use std::os::raw::c_int;

use pyo3::conversion::AsPyPointer;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple, PyType};
use pyo3::{ffi, intern, ToBorrowedObject};

use crate::build_tools::{py_error, SchemaDict};
use crate::errors::{as_internal, context, err_val_error, ErrorKind, InputValue, ValError, ValResult};
use crate::input::Input;

use super::{build_validator, BuildValidator, CombinedValidator, Extra, SlotsBuilder, Validator};

#[derive(Debug, Clone)]
pub struct ModelClassValidator {
    strict: bool,
    validator: Box<CombinedValidator>,
    class: Py<PyType>,
}

impl BuildValidator for ModelClassValidator {
    const EXPECTED_TYPE: &'static str = "model-class";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        slots_builder: &mut SlotsBuilder,
    ) -> PyResult<CombinedValidator> {
        let class: &PyType = schema.get_as_req("class")?;

        let model_schema_raw: &PyAny = schema.get_as_req("model")?;
        let (validator, model_schema) = build_validator(model_schema_raw, config, slots_builder)?;
        let model_type: String = model_schema.get_as_req("type")?;
        if &model_type != "model" {
            return py_error!("model-class expected a 'model' schema, got '{}'", model_type);
        }

        Ok(Self {
            // we don't use is_strict here since we don't wan validation to be strict in this case if
            // `config.strict` is set, only if this specific field is strict
            strict: schema.get_as("strict")?.unwrap_or(false),
            validator: Box::new(validator),
            class: class.into(),
        }
        .into())
    }
}

impl Validator for ModelClassValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        extra: &Extra,
        slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        let class = self.class.as_ref(py);
        if input.strict_model_check(class)? {
            Ok(input.to_py(py))
        } else if self.strict {
            err_val_error!(
                input_value = InputValue::InputRef(input),
                kind = ErrorKind::ModelType,
                context = context!("class_name" => self.get_name(py))
            )
        } else {
            let output = self.validator.validate(py, input, extra, slots)?;
            self.create_class(py, output).map_err(as_internal)
        }
    }

    fn validate_strict<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data dyn Input,
        _extra: &Extra,
        _slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        if input.strict_model_check(self.class.as_ref(py))? {
            Ok(input.to_py(py))
        } else {
            // errors from `validate_strict` are never used used, so we can keep this simple
            Err(ValError::LineErrors(vec![]))
        }
    }

    fn get_name(&self, py: Python) -> String {
        // Get the class's `__name__`, not using `class.name()` since it uses `__qualname__`
        // which is not what we want here
        let class = self.class.as_ref(py);
        let name_result: PyResult<&str> = match class.getattr(intern!(py, "__name__")) {
            Ok(name) => name.extract(),
            Err(e) => Err(e),
        };
        name_result.unwrap_or("ModelClass").to_string()
    }
}

impl ModelClassValidator {
    fn create_class(&self, py: Python, output: PyObject) -> PyResult<PyObject> {
        let t: &PyTuple = output.extract(py)?;
        let model_dict = t.get_item(0)?;
        let fields_set = t.get_item(1)?;

        let instance = self.class.call0(py)?;
        let instance_ptr = instance.as_ptr();

        force_setattr(instance_ptr, py, intern!(py, "__dict__"), model_dict)?;
        force_setattr(instance_ptr, py, intern!(py, "__fields_set__"), fields_set)?;

        Ok(instance)
    }
}

/// copied and modified from
/// https://github.com/PyO3/pyo3/blob/d2caa056e9aacc46374139ef491d112cb8af1a25/src/instance.rs#L587-L597
/// to use `PyObject_GenericSetAttr` thereby bypassing `__setattr__` methods on the instance,
/// see https://github.com/PyO3/pyo3/discussions/2321 for discussion
pub fn force_setattr<N, V>(obj: *mut ffi::PyObject, py: Python<'_>, attr_name: N, value: V) -> PyResult<()>
where
    N: ToPyObject,
    V: ToPyObject,
{
    attr_name.with_borrowed_ptr(py, move |attr_name| {
        value.with_borrowed_ptr(py, |value| unsafe {
            error_on_minusone(py, ffi::PyObject_GenericSetAttr(obj, attr_name, value))
        })
    })
}

// Defined here as it's not exported by pyo3
#[inline]
fn error_on_minusone(py: Python<'_>, result: c_int) -> PyResult<()> {
    if result != -1 {
        Ok(())
    } else {
        Err(PyErr::fetch(py))
    }
}
