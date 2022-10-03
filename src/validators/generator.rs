use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::SchemaDict;
use crate::errors::ValResult;
use crate::input::{GenericIterator, Input};
use crate::questions::Question;
use crate::recursion_guard::RecursionGuard;
use crate::ValidationError;

use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct GeneratorValidator {
    validator: Box<CombinedValidator>,
    name: String,
}

impl BuildValidator for GeneratorValidator {
    const EXPECTED_TYPE: &'static str = "generator";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let schema: &PyAny = schema.get_as_req(intern!(schema.py(), "items_schema"))?;
        let validator = Box::new(build_validator(schema, config, build_context)?);
        let name = format!("{}[{}]", Self::EXPECTED_TYPE, validator.get_name());
        Ok(Self { validator, name }.into())
    }
}

impl Validator for GeneratorValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let iterator = input.validate_iter()?;
        let validator = InternalValidator {
            validator: self.validator.clone(),
            slots: slots.to_vec(),
            data: extra.data.map(|d| d.into_py(py)),
            field: extra.field.map(|f| f.to_string()),
            strict: extra.strict,
            context: extra.context.map(|d| d.into_py(py)),
            recursion_guard: recursion_guard.clone(),
        };
        let v_iterator = ValidatorIterator { iterator, validator };
        Ok(v_iterator.into_py(py))
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn ask(&self, question: &Question) -> bool {
        self.validator.ask(question)
    }

    fn complete(&mut self, build_context: &BuildContext) -> PyResult<()> {
        self.validator.complete(build_context)
    }
}

#[pyclass]
#[derive(Debug, Clone)]
struct ValidatorIterator {
    iterator: GenericIterator,
    validator: InternalValidator,
}

#[pymethods]
impl ValidatorIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>, py: Python) -> Option<PyObject> {
        let validator_clone = &mut slf.validator.clone();
        match slf.iterator {
            GenericIterator::PyIterator(ref mut iter) => {
                match iter.as_ref(py).next() {
                    Some(next) => {
                        match next {
                            Ok(next) => Some(validator_clone.validate(py, next).unwrap()),
                            // Err(e) => Some(Err(e)),
                            Err(_) => None,
                        }
                    }
                    None => None,
                }
            }
            _ => todo!(),
        }
    }

    fn __repr__(&self) -> String {
        format!("ValidatorIter({:?})", self.validator.validator)
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

#[derive(Debug, Clone)]
pub struct InternalValidator {
    validator: Box<CombinedValidator>,
    slots: Vec<CombinedValidator>,
    data: Option<Py<PyDict>>,
    field: Option<String>,
    strict: Option<bool>,
    context: Option<PyObject>,
    recursion_guard: RecursionGuard,
}

impl InternalValidator {
    fn validate<'s, 'data>(&'s mut self, py: Python<'data>, input: &'data impl Input<'data>) -> PyResult<PyObject>
    where
        's: 'data,
    {
        let extra = Extra {
            data: self.data.as_ref().map(|data| data.as_ref(py)),
            field: self.field.as_deref(),
            strict: self.strict,
            context: self.context.as_ref().map(|data| data.as_ref(py)),
        };
        self.validator
            .validate(py, input, &extra, &self.slots, &mut self.recursion_guard)
            .map_err(|e| ValidationError::from_val_error(py, "ValidatorIterator".to_object(py), e))
    }
}
