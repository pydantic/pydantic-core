use std::borrow::Cow;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::{py_error, SchemaDict};
use crate::errors::{ValError, ValResult};
use crate::input::Input;
use crate::recursion_guard::RecursionGuard;
use crate::validators::build_validator;

use super::{BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
enum DefaultType {
    None,
    Default(PyObject),
    DefaultFactory(PyObject),
}

#[derive(Debug, Clone)]
enum OnError {
    Raise,
    Omit,
    FallbackOnDefault,
}

#[derive(Debug, Clone)]
pub struct WithDefaultValidator {
    default: DefaultType,
    on_error: OnError,
    validator: Box<CombinedValidator>,
    name: String,
}

impl BuildValidator for WithDefaultValidator {
    const EXPECTED_TYPE: &'static str = "default";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let default = match (
            schema.get_as(intern!(py, "default"))?,
            schema.get_as(intern!(py, "default_factory"))?,
        ) {
            (Some(_), Some(_)) => return py_error!("'default' and 'default_factory' cannot be used together"),
            (Some(default), None) => DefaultType::Default(default),
            (None, Some(default_factory)) => DefaultType::DefaultFactory(default_factory),
            (None, None) => DefaultType::None,
        };
        let on_error = match schema.get_as::<&str>(intern!(py, "on_error"))? {
            Some(on_error) => match on_error {
                "raise" => OnError::Raise,
                "omit" => OnError::Omit,
                "fallback_on_default" => {
                    if matches!(default, DefaultType::None) {
                        return py_error!("'on_error = {}' requires a `default` or `default_factory`", on_error);
                    }
                    OnError::FallbackOnDefault
                }
                _ => unreachable!(),
            },
            None => OnError::Raise,
        };

        let sub_schema: &PyAny = schema.get_as_req(intern!(schema.py(), "schema"))?;
        let validator = Box::new(build_validator(sub_schema, config, build_context)?);
        let name = format!("{}[{}]", Self::EXPECTED_TYPE, validator.get_name());

        Ok(Self {
            default,
            on_error,
            validator,
            name,
        }
        .into())
    }
}

impl Validator for WithDefaultValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        match self.validator.validate(py, input, extra, slots, recursion_guard) {
            Ok(v) => Ok(v),
            Err(e) => match self.on_error {
                OnError::Raise => Err(e),
                OnError::FallbackOnDefault => Ok(self.default_value(py)?.unwrap().as_ref().clone()),
                OnError::Omit => Err(ValError::Omit),
            },
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn complete(&mut self, build_context: &BuildContext) -> PyResult<()> {
        self.validator.complete(build_context)
    }
}

impl WithDefaultValidator {
    pub fn default_value(&self, py: Python) -> PyResult<Option<Cow<PyObject>>> {
        match self.default {
            DefaultType::Default(ref default) => Ok(Some(Cow::Borrowed(default))),
            DefaultType::DefaultFactory(ref default_factory) => Ok(Some(Cow::Owned(default_factory.call0(py)?))),
            DefaultType::None => Ok(None),
        }
    }

    pub fn has_default(&self) -> bool {
        !matches!(self.default, DefaultType::None)
    }

    pub fn omit_on_error(&self) -> bool {
        matches!(self.on_error, OnError::Omit)
    }
}
