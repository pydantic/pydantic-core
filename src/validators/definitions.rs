use std::cell::RefCell;

use ahash::HashSet;
use ahash::HashSetExt;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::definitions::DefinitionRef;
use crate::errors::ValResult;
use crate::input::Input;

use crate::tools::SchemaDict;

use super::{build_validator, BuildValidator, CombinedValidator, DefinitionsBuilder, ValidationState, Validator};

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
            let reference = schema_definition
                .extract::<&PyDict>()?
                .get_as_req::<String>(intern!(py, "ref"))?;
            let validator = build_validator(schema_definition, config, definitions)?;
            definitions.add_definition(reference, validator.inner)?;
        }

        let inner_schema: &PyAny = schema.get_as_req(intern!(py, "schema"))?;
        Ok(build_validator(inner_schema, config, definitions)?.inner)
    }
}

#[derive(Debug, Clone)]
pub struct DefinitionRefValidator {
    definition: DefinitionRef<CombinedValidator>,
}

impl DefinitionRefValidator {
    pub fn new(definition: DefinitionRef<CombinedValidator>) -> Self {
        Self { definition }
    }
}

impl BuildValidator for DefinitionRefValidator {
    const EXPECTED_TYPE: &'static str = "definition-ref";

    fn build(
        schema: &PyDict,
        _config: Option<&PyDict>,
        definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let schema_ref = schema.get_as_req(intern!(schema.py(), "schema_ref"))?;

        let definition = definitions.get_definition(schema_ref);
        Ok(Self::new(definition).into())
    }
}

impl_py_gc_traverse!(DefinitionRefValidator {});

impl Validator for DefinitionRefValidator {
    fn validate<'data>(
        &self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        state: &mut ValidationState,
    ) -> ValResult<'data, PyObject> {
        let mut token = state.descend();
        self.definition.get().unwrap().validate(py, input, token.get_state()?)
    }

    fn validate_assignment<'data>(
        &self,
        py: Python<'data>,
        obj: &'data PyAny,
        field_name: &'data str,
        field_value: &'data PyAny,
        state: &mut ValidationState,
    ) -> ValResult<'data, PyObject> {
        let mut token = state.descend();
        self.definition
            .get()
            .unwrap()
            .validate_assignment(py, obj, field_name, field_value, token.get_state()?)
    }

    fn different_strict_behavior(&self, ultra_strict: bool) -> bool {
        thread_local! {
            static RECURSION_SET: RefCell<Option<HashSet<usize>>> = RefCell::new(None);
        }

        let id = self as *const _ as usize;
        // have to unwrap here, because we can't return an error from this function, should be okay
        let validator: &CombinedValidator = self.definition.get().unwrap();
        if RECURSION_SET.with(
            |set: &RefCell<Option<std::collections::HashSet<usize, ahash::RandomState>>>| {
                set.borrow_mut().get_or_insert_with(HashSet::new).insert(id)
            },
        ) {
            let different_strict_behavior = validator.different_strict_behavior(ultra_strict);
            RECURSION_SET.with(|set| set.borrow_mut().get_or_insert_with(HashSet::new).remove(&id));
            different_strict_behavior
        } else {
            false
        }
    }

    fn get_name(&self) -> &str {
        self.definition.get_or_init_name(|v| v.get_name().into())
    }

    fn complete(&self) -> PyResult<()> {
        Ok(())
    }
}
