use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::build_tools::{py_error, SchemaDict};
use crate::errors::ValResult;
use crate::input::Input;
use crate::questions::Question;
use crate::recursion_guard::RecursionGuard;

use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct ChainValidator {
    steps: Vec<CombinedValidator>,
    name: String,
}

impl BuildValidator for ChainValidator {
    const EXPECTED_TYPE: &'static str = "chain";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let steps: Vec<CombinedValidator> = schema
            .get_as_req::<&PyList>(intern!(schema.py(), "steps"))?
            .iter()
            .map(|step| build_validator_steps(step, config, build_context))
            .collect::<PyResult<Vec<Vec<CombinedValidator>>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<CombinedValidator>>();

        match steps.len() {
            0 => py_error!("One or more steps required for a chain validator"),
            1 => {
                let step = steps.into_iter().next().unwrap();
                Ok(step)
            }
            _ => {
                let descr = steps.iter().map(|v| v.get_name()).collect::<Vec<_>>().join(",");

                Ok(Self {
                    steps,
                    name: format!("{}[{}]", Self::EXPECTED_TYPE, descr),
                }
                .into())
            }
        }
    }
}

// either a vec of the steps from a nested `ChainValidator`, or a length-1 vec containing the validator
// to be flattened into `steps` above
fn build_validator_steps<'a>(
    step: &'a PyAny,
    config: Option<&'a PyDict>,
    build_context: &mut BuildContext,
) -> PyResult<Vec<CombinedValidator>> {
    let validator = build_validator(step, config, build_context)?;
    if let CombinedValidator::Chain(chain_validator) = validator {
        Ok(chain_validator.steps)
    } else {
        Ok(vec![validator])
    }
}

impl Validator for ChainValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let mut steps_iter = self.steps.iter();
        let first_step = steps_iter.next().unwrap();
        let value = first_step
            .validate(py, input, extra, slots, recursion_guard)
            .map_err(|err| err.with_outer_location(self.name.clone().into()))?;

        steps_iter
            .try_fold(value, |v, step| {
                step.validate(py, v.into_ref(py), extra, slots, recursion_guard)
            })
            .map_err(|err| err.with_outer_location(self.name.clone().into()))
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn ask(&self, question: &Question) -> bool {
        self.steps.iter().all(|v| v.ask(question))
    }

    fn complete(&mut self, build_context: &BuildContext) -> PyResult<()> {
        self.steps.iter_mut().try_for_each(|v| v.complete(build_context))
    }
}

// ---------------------------

#[derive(Debug, Clone)]
pub struct Chain2Validator {
    validator1: Box<CombinedValidator>,
    validator2: Box<CombinedValidator>,
    name: String,
}

impl BuildValidator for Chain2Validator {
    const EXPECTED_TYPE: &'static str = "chain2";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let schema1: &PyAny = schema.get_as_req(intern!(schema.py(), "schema1"))?;
        let validator1 = Box::new(build_validator(schema1, config, build_context)?);

        let schema2: &PyAny = schema.get_as_req(intern!(schema.py(), "schema2"))?;
        let validator2 = Box::new(build_validator(schema2, config, build_context)?);

        let name = format!(
            "{}[{}, {}]",
            Self::EXPECTED_TYPE,
            validator1.get_name(),
            validator2.get_name()
        );

        Ok(Self {
            validator1,
            validator2,
            name,
        }
        .into())
    }
}

impl Validator for Chain2Validator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let value = self
            .validator1
            .validate(py, input, extra, slots, recursion_guard)
            .map_err(|err| err.with_outer_location(self.name.clone().into()))?;
        return self
            .validator2
            .validate(py, value.into_ref(py), extra, slots, recursion_guard)
            .map_err(|err| err.with_outer_location(self.name.clone().into()));
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn ask(&self, question: &Question) -> bool {
        self.validator1.ask(question) && self.validator2.ask(question)
    }

    fn complete(&mut self, build_context: &BuildContext) -> PyResult<()> {
        self.validator1.complete(build_context)?;
        self.validator2.complete(build_context)
    }
}

#[derive(Debug, Clone)]
pub struct Chain3Validator {
    validator1: Box<CombinedValidator>,
    validator2: Box<CombinedValidator>,
    validator3: Box<CombinedValidator>,
    name: String,
}

impl BuildValidator for Chain3Validator {
    const EXPECTED_TYPE: &'static str = "chain3";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let schema1: &PyAny = schema.get_as_req(intern!(schema.py(), "schema1"))?;
        let validator1 = Box::new(build_validator(schema1, config, build_context)?);

        let schema2: &PyAny = schema.get_as_req(intern!(schema.py(), "schema2"))?;
        let validator2 = Box::new(build_validator(schema2, config, build_context)?);

        let schema3: &PyAny = schema.get_as_req(intern!(schema.py(), "schema3"))?;
        let validator3 = Box::new(build_validator(schema3, config, build_context)?);

        let name = format!(
            "{}[{}, {}, {}]",
            Self::EXPECTED_TYPE,
            validator1.get_name(),
            validator2.get_name(),
            validator3.get_name()
        );

        Ok(Self {
            validator1,
            validator2,
            validator3,
            name,
        }
        .into())
    }
}

impl Validator for Chain3Validator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let value = self
            .validator1
            .validate(py, input, extra, slots, recursion_guard)
            .map_err(|err| err.with_outer_location(self.name.clone().into()))?;
        let value = self
            .validator2
            .validate(py, value.into_ref(py), extra, slots, recursion_guard)
            .map_err(|err| err.with_outer_location(self.name.clone().into()))?;

        self.validator3
            .validate(py, value.into_ref(py), extra, slots, recursion_guard)
            .map_err(|err| err.with_outer_location(self.name.clone().into()))
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn ask(&self, question: &Question) -> bool {
        self.validator1.ask(question) && self.validator2.ask(question) && self.validator3.ask(question)
    }

    fn complete(&mut self, build_context: &BuildContext) -> PyResult<()> {
        self.validator1.complete(build_context)?;
        self.validator2.complete(build_context)?;
        self.validator3.complete(build_context)
    }
}
