use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::build_tools::{py_err, SchemaDict};
use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::Input;
use crate::recursion_guard::RecursionGuard;

use super::{BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct LiteralValidator {
    expected_py: Py<PyDict>,
    expected_repr: String,
    name: String,
}

impl BuildValidator for LiteralValidator {
    const EXPECTED_TYPE: &'static str = "literal";

    fn build(
        schema: &PyDict,
        _config: Option<&PyDict>,
        _build_context: &mut BuildContext<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let expected: &PyList = schema.get_as_req(intern!(schema.py(), "expected"))?;
        if expected.is_empty() {
            return py_err!(r#""expected" should have length > 0"#);
        }
        let py = expected.py();
        // Literal[...] only supports int, str, bytes or enums, all of which can be hashed
        let expected_py = PyDict::new(py);
        let mut repr_args: Vec<String> = Vec::new();
        for item in expected.iter() {
            repr_args.push(item.repr()?.extract()?);
            expected_py.set_item(item, item)?;
        }
        let (expected_repr, name) = expected_repr_name(repr_args, "literal");
        Ok(CombinedValidator::Literal(Self {
            expected_py: expected_py.into(),
            expected_repr,
            name,
        }))
    }
}

impl Validator for LiteralValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        _extra: &Extra,
        _slots: &'data [CombinedValidator],
        _recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        if let Some(v) = self.expected_py.as_ref(py).get_item(input) {
            return Ok(v.into());
        }
        Err(ValError::new(
            ErrorType::LiteralError {
                expected: self.expected_repr.clone(),
            },
            input,
        ))
    }

    fn different_strict_behavior(
        &self,
        _build_context: Option<&BuildContext<CombinedValidator>>,
        ultra_strict: bool,
    ) -> bool {
        !ultra_strict
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn complete(&mut self, _build_context: &BuildContext<CombinedValidator>) -> PyResult<()> {
        Ok(())
    }
}

pub fn expected_repr_name(mut repr_args: Vec<String>, base_name: &'static str) -> (String, String) {
    let name = format!("{base_name}[{}]", repr_args.join(","));
    // unwrap is okay since we check the length in build at the top of this file
    let last_repr = repr_args.pop().unwrap();
    let repr = if repr_args.is_empty() {
        last_repr
    } else {
        format!("{} or {last_repr}", repr_args.join(", "))
    };
    (repr, name)
}
