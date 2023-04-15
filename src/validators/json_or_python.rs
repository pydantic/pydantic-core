use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::SchemaDict;
use crate::errors::ValResult;
use crate::input::{Input, InputType};
use crate::recursion_guard::RecursionGuard;

use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct JsonOrPython {
    json: Box<CombinedValidator>,
    python: Box<CombinedValidator>,
    name: String,
}

impl BuildValidator for JsonOrPython {
    const EXPECTED_TYPE: &'static str = "json-or-python";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let json_schema: &PyDict = schema.get_as_req(intern!(py, "json_schema"))?;
        let python_schema: &PyDict = schema.get_as_req(intern!(py, "python_schema"))?;

        let json = build_validator(json_schema, config, build_context)?;
        let python = build_validator(python_schema, config, build_context)?;

        let name = format!(
            "{}[json={},python={}]",
            Self::EXPECTED_TYPE,
            json.get_name(),
            python.get_name(),
        );
        Ok(Self {
            json: Box::new(json),
            python: Box::new(python),
            name,
        }
        .into())
    }
}

impl Validator for JsonOrPython {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        match input.get_type() {
            InputType::Json => self.json.validate(py, input, extra, slots, recursion_guard),
            // String gets treated the same as Json
            InputType::String => self.json.validate(py, input, extra, slots, recursion_guard),
            InputType::Python => self.python.validate(py, input, extra, slots, recursion_guard),
        }
    }

    fn different_strict_behavior(
        &self,
        build_context: Option<&BuildContext<CombinedValidator>>,
        ultra_strict: bool,
    ) -> bool {
        self.json.different_strict_behavior(build_context, ultra_strict)
            || self.python.different_strict_behavior(build_context, ultra_strict)
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn complete(&mut self, build_context: &BuildContext<CombinedValidator>) -> PyResult<()> {
        self.json.complete(build_context)?;
        self.python.complete(build_context)
    }
}
