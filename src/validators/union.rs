use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyType};

use super::{build_validator, Extra, ValResult, Validator};
use crate::build_macros::{dict_get, dict_get_required};
use crate::errors::{ValError, ValLineError};
use crate::input::Input;

#[derive(Debug, Clone)]
pub struct UnionValidator {
    validators: Vec<Box<dyn Validator>>,
    validator_classes: Vec<Py<PyType>>,
}

impl UnionValidator {
    pub const EXPECTED_TYPE: &'static str = "union";
}

impl Validator for UnionValidator {
    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<Box<dyn Validator>> {
        let mut validators: Vec<Box<dyn Validator>> = vec![];
        let mut validator_classes: Vec<Py<PyType>> = vec![];
        let sub_schemas: &PyList = dict_get_required!(schema, "schemas", &PyList)?;
        for sub_schema_ in sub_schemas.iter() {
            let sub_schema: &PyDict = sub_schema_.extract()?;
            let validator = build_validator(sub_schema, config)?;
            validators.push(validator);
            if let Some(class) = dict_get!(sub_schema, "class", &PyType) {
                validator_classes.push(class.into());
            }
        }

        Ok(Box::new(Self {
            validators,
            validator_classes,
        }))
    }

    fn validate(&self, py: Python, input: &dyn Input, extra: &Extra) -> ValResult<PyObject> {
        let mut errors: Vec<ValLineError> = Vec::with_capacity(self.validators.len());

        // 1st pass: check if the value is an exact instance of one of the Union types
        for class in &self.validator_classes {
            if input.is_direct_instance_of(class.as_ref(py))? {
                return Ok(input.to_py(py));
            }
        }

        // 3rd pass: check if the value can be coerced into one of the Union types
        for validator in &self.validators {
            let line_errors = match validator.validate(py, input, extra) {
                Ok(item) => return Ok(item),
                Err(ValError::LineErrors(line_errors)) => line_errors,
                Err(err) => return Err(err),
            };

            // let loc = vec![LocItem::I(index)];
            for err in line_errors {
                // errors.push(err.prefix_location(&loc));
                errors.push(err);
            }
        }
        Err(ValError::LineErrors(errors))
    }

    fn clone_dyn(&self) -> Box<dyn Validator> {
        Box::new(self.clone())
    }
}
