use std::str::FromStr;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use crate::build_tools::py_schema_err;
use crate::errors::ValResult;
use crate::input::Input;
use crate::tools::SchemaDict;

use super::validation_state::ValidationState;
use super::{build_validator, BuildValidator, CombinedValidator, DefinitionsBuilder, Validator};

#[derive(Debug)]
enum VarKwargsMode {
    Single,
    TypedDict,
}

impl FromStr for VarKwargsMode {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "single" => Ok(Self::Single),
            "typed_dict" => Ok(Self::TypedDict),
            s => py_schema_err!("Invalid var_kwargs mode: `{}`, expected `single` or `typed_dict`", s),
        }
    }
}

#[derive(Debug)]
pub struct VarKwargsValidator {
    mode: VarKwargsMode,
    validator: Box<CombinedValidator>,
}

impl BuildValidator for VarKwargsValidator {
    const EXPECTED_TYPE: &'static str = "var_kwargs";

    fn build(
        schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();

        let py_mode: Bound<PyString> = schema.get_as_req(intern!(py, "mode"))?;
        let mode = VarKwargsMode::from_str(py_mode.to_string().as_str())?;

        let validator_schema: Bound<PyDict> = schema.get_as_req(intern!(py, "schema"))?;

        Ok(Self {
            mode,
            validator: Box::new(build_validator(&validator_schema, config, definitions)?),
        }
        .into())
    }
}

impl_py_gc_traverse!(VarKwargsValidator { mode, validator });

impl Validator for VarKwargsValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        match self.mode {
            VarKwargsMode::Single => {
                // TODO
            }
            VarKwargsMode::TypedDict => {
                // TODO
            }
        }
    }
}
