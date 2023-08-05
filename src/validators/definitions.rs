use std::fmt::Debug;
use std::sync::OnceLock;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::definitions::DefinitionRef;
use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::Input;

use crate::recursion_guard::RecursionGuard;
use crate::tools::SchemaDict;

use super::{build_validator, BuildValidator, CombinedValidator, DefinitionsBuilder, Extra, Validator};

#[derive(Debug, Clone)]
pub struct DefinitionsValidatorBuilder;

impl BuildValidator for DefinitionsValidatorBuilder {
    const EXPECTED_TYPE: &'static str = "definitions";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();

        let schema_definitions: &PyList = schema.get_as_req(intern!(py, "definitions"))?;

        for schema_definition in schema_definitions {
            build_validator(schema_definition, config, definitions)?;
            // no need to store the validator here, it has already been stored in definitions if necessary
        }

        let inner_schema: &PyAny = schema.get_as_req(intern!(py, "schema"))?;
        build_validator(inner_schema, config, definitions)
    }
}

#[derive(Debug)]
pub struct DefinitionRefValidator {
    definition: DefinitionRef<CombinedValidator>,
}

impl DefinitionRefValidator {
    pub fn new(definition: DefinitionRef<CombinedValidator>) -> Self {
        Self {
            definition,
            different_strict_behaviour: OnceLock::new(),
            different_ultra_strict_behaviour: OnceLock::new(),
        }
    }
}

impl BuildValidator for DefinitionRefValidator {
    const EXPECTED_TYPE: &'static str = "definition-ref";

    fn build(
        schema: &PyDict,
        _config: Option<&PyDict>,
        definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let schema_ref: String = schema.get_as_req(intern!(schema.py(), "schema_ref"))?;

        let definition = definitions.get_definition(&schema_ref).clone();

        Ok(Self { definition }.into())
    }
}

// NB it is NOT correct to traverse the definition here; doing so may lead to circular references.
// Instead leave the traversal to the definitions in the top-level schema validator.
impl_py_gc_traverse!(DefinitionRefValidator {});

impl Validator for DefinitionRefValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let validator = self.definition.get().unwrap();
        if let Some(id) = input.identity() {
            if recursion_guard.contains_or_insert(id, self.definition.id()) {
                // we don't remove id here, we leave that to the validator which originally added id to `recursion_guard`
                Err(ValError::new(ErrorType::RecursionLoop, input))
            } else {
                if recursion_guard.incr_depth() {
                    return Err(ValError::new(ErrorType::RecursionLoop, input));
                }
                let output = validator.validate(py, input, extra, recursion_guard);
                recursion_guard.remove(id, self.definition.id());
                recursion_guard.decr_depth();
                output
            }
        } else {
            validator.validate(py, input, extra, recursion_guard)
        }
    }

    fn validate_assignment<'s, 'data: 's>(
        &'s self,
        py: Python<'data>,
        obj: &'data PyAny,
        field_name: &'data str,
        field_value: &'data PyAny,
        extra: &Extra,
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let validator = self.definition.get().unwrap();
        if let Some(id) = obj.identity() {
            if recursion_guard.contains_or_insert(id, self.definition.id()) {
                // we don't remove id here, we leave that to the validator which originally added id to `recursion_guard`
                Err(ValError::new(ErrorType::RecursionLoop, obj))
            } else {
                if recursion_guard.incr_depth() {
                    return Err(ValError::new(ErrorType::RecursionLoop, obj));
                }
                let output = validator.validate_assignment(py, obj, field_name, field_value, extra, recursion_guard);
                recursion_guard.remove(id, self.definition.id());
                recursion_guard.decr_depth();
                output
            }
        } else {
            validator.validate_assignment(py, obj, field_name, field_value, extra, recursion_guard)
        }
    }

    fn different_strict_behavior(&self, ultra_strict: bool) -> bool {
        // have to unwrap here, because we can't return an error from this function, should be okay
        let validator = self.definition.get().unwrap();
        validator.different_strict_behavior(ultra_strict)
    }

    fn get_name(&self) -> &str {
        self.definition.get().map_or("...", |validator| validator.get_name())
    }
}
