use pyo3::types::PyDict;
use pyo3::{intern, prelude::*};

use crate::build_tools::py_schema_err;
use crate::definitions::DefinitionsBuilder;
use crate::errors::ValResult;
use crate::input::Input;
use crate::tools::SchemaDict;
use crate::SchemaValidator;

use super::{BuildValidator, CombinedValidator, ValidationState, Validator};

#[derive(Debug)]
pub struct PrecompiledValidator {
    validator: Py<SchemaValidator>,
}

impl BuildValidator for PrecompiledValidator {
    const EXPECTED_TYPE: &'static str = "precompiled";

    fn build(
        schema: &PyDict,
        _config: Option<&PyDict>,
        _definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let sub_schema: &PyAny = schema.get_as_req(intern!(py, "schema"))?;
        let validator: PyRef<SchemaValidator> = schema.get_as_req(intern!(py, "validator"))?;

        // TODO DEBUG THIS LATER
        // if !validator.schema.is(sub_schema) {
        //     return py_schema_err!("precompiled schema mismatch");
        // }

        Ok(CombinedValidator::Precompiled(PrecompiledValidator {
            validator: validator.into(),
        }))
    }
}

impl_py_gc_traverse!(PrecompiledValidator { validator });

impl Validator for PrecompiledValidator {
    fn validate<'data>(
        &self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        state: &mut ValidationState,
    ) -> ValResult<'data, PyObject> {
        self.validator.get().validator.validate(py, input, state)
    }

    fn different_strict_behavior(&self, ultra_strict: bool) -> bool {
        self.validator.get().validator.different_strict_behavior(ultra_strict)
    }

    fn get_name(&self) -> &str {
        self.validator.get().validator.get_name()
    }

    fn complete(&self) -> PyResult<()> {
        // No need to complete a precompiled validator
        Ok(())
    }
}
