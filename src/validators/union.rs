use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyType};

use super::{build_validator, Extra, ValResult, Validator};
use crate::build_macros::{dict_get, dict_get_required};
use crate::errors::{LocItem, ValError, ValLineError};
use crate::input::Input;

#[derive(Debug, Clone)]
struct UnionChoice {
    validator: Box<dyn Validator>,
    class: Option<Py<PyType>>,
}

#[derive(Debug, Clone)]
pub struct UnionValidator {
    choices: Vec<UnionChoice>,
}

impl UnionValidator {
    pub const EXPECTED_TYPE: &'static str = "union";
}

impl Validator for UnionValidator {
    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<Box<dyn Validator>> {
        let mut choices: Vec<UnionChoice> = vec![];
        let choice_schema: &PyList = dict_get_required!(schema, "choices", &PyList)?;
        for choice in choice_schema.iter() {
            let choice_dict: &PyDict = choice.extract()?;
            let validator = build_validator(choice_dict, config)?;
            choices.push(UnionChoice {
                validator,
                class: dict_get!(choice_dict, "class", &PyType).map(|class| class.into()),
            });
        }

        Ok(Box::new(Self { choices }))
    }

    fn validate(&self, py: Python, input: &dyn Input, extra: &Extra) -> ValResult<PyObject> {
        // 1st pass: check if the value is an exact instance of one of the Union types
        for class in self.choices.iter().flat_map(|c| &c.class) {
            if input.is_direct_instance_of(class.as_ref(py))? {
                return Ok(input.to_py(py));
            }
        }
        let mut errors: Vec<ValLineError> = Vec::with_capacity(self.choices.len());

        // 3rd pass: check if the value can be coerced into one of the Union types
        for choice in &self.choices {
            let line_errors = match choice.validator.validate(py, input, extra) {
                Ok(item) => return Ok(item),
                Err(ValError::LineErrors(line_errors)) => line_errors,
                Err(err) => return Err(err),
            };

            let loc = vec![LocItem::S(choice.validator.get_name())];
            for err in line_errors {
                errors.push(err.prefix_location(&loc));
            }
        }
        Err(ValError::LineErrors(errors))
    }

    fn get_name(&self) -> String {
        Self::EXPECTED_TYPE.to_string()
    }

    fn clone_dyn(&self) -> Box<dyn Validator> {
        Box::new(self.clone())
    }
}
