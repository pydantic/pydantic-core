use mail_parser::Addr;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use super::{BuildContext, BuildValidator, CombinedValidator, Extra, Validator};
use crate::build_tools::is_strict;
use crate::errors::ValResult;
use crate::input::{Input, NameEmail};
use crate::recursion_guard::RecursionGuard;

#[derive(Debug, Clone)]
pub struct NameEmailValidator {
    strict: bool,
}

impl BuildValidator for NameEmailValidator {
    const EXPECTED_TYPE: &'static str = "name-email";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        _build_context: &mut BuildContext<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        Ok(Self {
            strict: is_strict(schema, config)?,
        }
        .into())
    }
}

impl Validator for NameEmailValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        _slots: &'data [CombinedValidator],
        _recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let email_name = match input.validate_name_email(extra.strict.unwrap_or(self.strict)) {
            Ok(email_name) => email_name,
            // if the date error was an internal error, return that immediately
            Err(err) => return Err(err),
        };

        let NameEmail::Raw(Addr { name, address }) = email_name;
        match (name, address) {
            // TODO: to_mut?
            (Some(_name), Some(mut address)) => return Ok(PyString::new(py, address.to_mut()).into_py(py)),
            // TODO: fix
            _ => return Ok(PyString::new(py, "").into_py(py)),
        }
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}
