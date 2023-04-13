use ahash::{HashMap, HashMapExt};
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
    expected_int: Option<HashMap<i64, Py<PyAny>>>,
    expected_str: Option<HashMap<String, Py<PyAny>>>,
    expected_py: Option<Py<PyDict>>,
    expected_repr: String,
    name: String,
}

fn extract_int_strict(py: Python, item: &PyAny) -> Option<i64> {
    if item.get_type().is(1.to_object(py).as_ref(py).get_type()) {
        return Some(item.extract().unwrap());
    }
    None
}

fn extract_str_strict(py: Python, item: &PyAny) -> Option<String> {
    if item.get_type().is(intern!(py, "a").get_type()) {
        return Some(item.extract().unwrap());
    }
    None
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
        let mut expected_int = HashMap::new();
        let mut expected_str = HashMap::new();
        let expected_py = PyDict::new(py);
        let mut repr_args: Vec<String> = Vec::new();
        for item in expected.iter() {
            repr_args.push(item.repr()?.extract()?);
            if let Some(int) = extract_int_strict(py, item) {
                expected_int.insert(int, item.into());
            } else if let Some(string) = extract_str_strict(py, item) {
                expected_str.insert(string.to_string(), item.into());
            } else {
                expected_py.set_item(item, item)?;
            }
        }
        let (expected_repr, name) = expected_repr_name(repr_args, "literal");
        Ok(CombinedValidator::Literal(Self {
            expected_int: (!expected_int.is_empty()).then_some(expected_int),
            expected_str: (!expected_str.is_empty()).then_some(expected_str),
            expected_py: (!expected_py.is_empty()).then_some(expected_py.into()),
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
        if let Some(expected_ints) = &self.expected_int {
            if let Some(int) = extract_int_strict(py, input.to_object(py).as_ref(py)) {
                if let Some(value) = expected_ints.get(&int) {
                    return Ok(value.clone());
                }
            }
        }
        if let Some(expected_strings) = &self.expected_str {
            if let Some(string) = extract_str_strict(py, input.to_object(py).as_ref(py)) {
                if let Some(value) = expected_strings.get(&string) {
                    return Ok(value.clone());
                }
            }
        }
        if let Some(expected_py) = &self.expected_py {
            if let Some(v) = expected_py.as_ref(py).get_item(input) {
                return Ok(v.into());
            }
        };
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
