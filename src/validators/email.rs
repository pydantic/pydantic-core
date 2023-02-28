use email_address::EmailAddress;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::str::FromStr;

use crate::build_tools::is_strict;
use crate::email::PyEmail;
use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::Input;
use crate::recursion_guard::RecursionGuard;

use super::{BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct EmailValidator {
    strict: bool,
}

impl EmailValidator {
    fn get_email<'s, 'data>(&'s self, input: &'data impl Input<'data>, strict: bool) -> ValResult<'data, EmailAddress> {
        match input.validate_str(strict) {
            Ok(either_str) => {
                let cow = either_str.as_cow()?;
                let email_str = cow.as_ref();
                parse_email(email_str, input)
            }
            Err(_) => {
                if let Some(py_email) = input.input_as_email() {
                    let lib_email = py_email.into_email();
                    Ok(lib_email)
                } else {
                    Err(ValError::new(ErrorType::EmailType, input))
                }
            }
        }
    }
}

fn parse_email<'email, 'input>(
    email_str: &'email str,
    input: &'input impl Input<'input>,
) -> ValResult<'input, EmailAddress> {
    EmailAddress::from_str(email_str)
        .map_err(move |e| ValError::new(ErrorType::EmailParsing { error: e.to_string() }, input))
}

impl BuildValidator for EmailValidator {
    const EXPECTED_TYPE: &'static str = "email";

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

impl Validator for EmailValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        _slots: &'data [CombinedValidator],
        _recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let lib_email = self.get_email(input, extra.strict.unwrap_or(self.strict))?;

        Ok(PyEmail::new(lib_email).into_py(py))
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}
