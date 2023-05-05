use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use uuid::Uuid;

use crate::build_tools::{is_strict, SchemaDict};
use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::Input;
use crate::recursion_guard::RecursionGuard;
use crate::uuid::PyUuid;

use super::{BuildValidator, CombinedValidator, Definitions, DefinitionsBuilder, Extra, Validator};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UuidValidator {
    strict: bool,
    version: Option<usize>,
}

impl BuildValidator for UuidValidator {
    const EXPECTED_TYPE: &'static str = "uuid";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        _definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        Ok(Self {
            strict: is_strict(schema, config)?,
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
        extra: &Extra,
        _definitions: &'data Definitions<CombinedValidator>,
        _recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let lib_uuid = self.get_uuid(input, extra.strict.unwrap_or(self.strict))?;
        Ok(PyUuid::new(lib_uuid).into_py(py))
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
    fn get_uuid<'s, 'data>(&'s self, input: &'data impl Input<'data>, strict: bool) -> ValResult<'data, Uuid> {
        match input.validate_str(strict) {
            Ok(either_uuid) => {
                let cow = either_uuid.as_cow()?;
                let uuid_str = cow.as_ref();
                match Uuid::parse_str(uuid_str) {
                    Ok(lib_uuid) => Ok(lib_uuid),
                    Err(e) => Err(ValError::new(ErrorType::UuidParsing { error: e.to_string() }, input)),
                }
            }
            Err(_) => {
                if let Some(py_uuid) = input.input_as_uuid() {
                    let lib_uuid = py_uuid.into_uuid();
                    self.check_version(input, lib_uuid)?;
                    Ok(lib_uuid)
                } else {
                    Err(ValError::new(ErrorType::UuidType, input))
                }
            }
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
