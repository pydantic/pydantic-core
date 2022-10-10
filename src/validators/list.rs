use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::SchemaDict;
use crate::errors::ValResult;
use crate::input::{GenericCollection, Input};
use crate::recursion_guard::RecursionGuard;

use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct ListValidator {
    strict: bool,
    allow_any_iter: bool,
    item_validator: Option<Box<CombinedValidator>>,
    size_range: (Option<usize>, Option<usize>),
    name: String,
}

pub fn get_items_schema(
    schema: &PyDict,
    config: Option<&PyDict>,
    build_context: &mut BuildContext,
) -> PyResult<Option<Box<CombinedValidator>>> {
    match schema.get_item(pyo3::intern!(schema.py(), "items_schema")) {
        Some(d) => {
            let validator = build_validator(d, config, build_context)?;
            match validator {
                CombinedValidator::Any(_) => Ok(None),
                _ => Ok(Some(Box::new(validator))),
            }
        }
        None => Ok(None),
    }
}

macro_rules! generic_collection_build {
    () => {
        super::list::generic_collection_build!("{}[{}]", Self::EXPECTED_TYPE);
    };
    ($name_template:literal, $name:expr) => {
        fn build(
            schema: &PyDict,
            config: Option<&PyDict>,
            build_context: &mut BuildContext,
        ) -> PyResult<CombinedValidator> {
            let py = schema.py();
            let item_validator = super::list::get_items_schema(schema, config, build_context)?;
            let inner_name = item_validator.as_ref().map(|v| v.get_name()).unwrap_or("any");
            let name = format!($name_template, $name, inner_name);
            let min_length = schema.get_as(pyo3::intern!(py, "min_length"))?;
            let max_length = schema.get_as(pyo3::intern!(py, "max_length"))?;
            Ok(Self {
                strict: crate::build_tools::is_strict(schema, config)?,
                item_validator,
                size_range: (min_length, max_length),
                name,
            }
            .into())
        }
    };
}
pub(crate) use generic_collection_build;

impl BuildValidator for ListValidator {
    const EXPECTED_TYPE: &'static str = "list";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let item_validator = get_items_schema(schema, config, build_context)?;
        let inner_name = item_validator.as_ref().map(|v| v.get_name()).unwrap_or("any");
        let name = format!("{}[{}]", Self::EXPECTED_TYPE, inner_name);
        let min_length = schema.get_as(pyo3::intern!(py, "min_length"))?;
        let max_length = schema.get_as(pyo3::intern!(py, "max_length"))?;
        Ok(Self {
            strict: crate::build_tools::is_strict(schema, config)?,
            allow_any_iter: schema.get_as(pyo3::intern!(py, "allow_any_iter"))?.unwrap_or(false),
            item_validator,
            size_range: (min_length, max_length),
            name,
        }
        .into())
    }
}

impl Validator for ListValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let seq = input.validate_list(extra.strict.unwrap_or(self.strict), self.allow_any_iter)?;

        let (capacity, check_max_length) = seq.pre_check(self.size_range, input, false)?;

        let output = match self.item_validator {
            Some(ref v) => {
                seq.validate_to_vec(py, input, capacity, check_max_length, v, extra, slots, recursion_guard)?
            }
            None => match seq {
                GenericCollection::List(list) => return Ok(list.into_py(py)),
                _ => seq.to_vec(py, input, check_max_length)?,
            },
        };
        Ok(output.into_py(py))
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn complete(&mut self, build_context: &BuildContext) -> PyResult<()> {
        match self.item_validator {
            Some(ref mut v) => v.complete(build_context),
            None => Ok(()),
        }
    }
}
