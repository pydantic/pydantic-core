use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use uuid::Uuid;

use crate::build_tools::SchemaDict;
use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::Input;
use crate::recursion_guard::RecursionGuard;
use crate::uuid::PyUuid;

use super::{BuildValidator, CombinedValidator, Definitions, DefinitionsBuilder, Extra, Validator};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UuidValidator {
    version: Option<usize>,
}

impl BuildValidator for UuidValidator {
    const EXPECTED_TYPE: &'static str = "uuid";

    fn build(
        schema: &PyDict,
        _config: Option<&PyDict>,
        _definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        Ok(Self {
            version: schema.get_as(intern!(schema.py(), "version"))?,
        }
        .into())
    }
}

impl Validator for UuidValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        _extra: &Extra,
        _definitions: &'data Definitions<CombinedValidator>,
        _recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        match self.get_uuid(input) {
            Ok(lib_uuid) => Ok(PyUuid::new(lib_uuid).into_py(py)),
            Err(error_type) => Err(error_type),
        }
    }

    fn different_strict_behavior(
        &self,
        _definitions: Option<&DefinitionsBuilder<CombinedValidator>>,
        _ultra_strict: bool,
    ) -> bool {
        false
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }

    fn complete(&mut self, _definitions: &DefinitionsBuilder<CombinedValidator>) -> PyResult<()> {
        Ok(())
    }
}

#[allow(dead_code)]
impl UuidValidator {
    fn get_uuid<'s, 'data>(&'s self, input: &'data impl Input<'data>) -> ValResult<'data, Uuid> {
        if let Some(py_uuid) = input.input_as_uuid() {
            let lib_uuid = py_uuid.into_uuid();
            self.check_version(input, lib_uuid)?;
            Ok(lib_uuid)
        } else {
            Err(ValError::new(ErrorType::UuidType, input))
        }
    }

    fn check_version<'s, 'data>(&self, input: &'data impl Input<'data>, uuid: Uuid) -> ValResult<'data, ()> {
        if let Some(schema_version) = self.version {
            let version = uuid.get_version_num();
            if schema_version == version {
                return Ok(());
            } else {
                return Err(ValError::new(
                    ErrorType::UuidVersionMismatch {
                        version,
                        schema_version,
                    },
                    input,
                ));
            }
        }
        Ok(())
    }
}
