use std::fmt;

use pyo3::prelude::*;
use pyo3::types::PyDict;

pub type Context = Option<Vec<(String, ContextValue)>>;

pub fn new_context<I: IntoIterator<Item = (String, ContextValue)>>(raw: I) -> Context {
    Some(raw.into_iter().collect())
}

pub fn render_message(context: &Context, template: String) -> String {
    match context {
        Some(ref ctx) => {
            let mut rendered = template;
            for (key, value) in ctx {
                rendered = rendered.replace(&format!("{{{}}}", key), &value.to_string());
            }
            rendered
        }
        None => template,
    }
}

pub fn context_as_py(context: &Context, py: Python) -> PyResult<PyObject> {
    match context {
        Some(ref ctx) => {
            let dict = PyDict::new(py);
            for (key, value) in ctx {
                dict.set_item(key, value)?;
            }
            Ok(dict.into_py(py))
        }
        None => Ok(py.None().into_py(py)),
    }
}

// fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//     match self.0 {
//         Some(ref ctx) => {
//             let loc = ctx
//                 .iter()
//                 .map(|(k, v)| format!("{}: {}", k, v))
//                 .collect::<Vec<_>>()
//                 .join(", ");
//             write!(f, "{{{}}}", loc)
//         }
//         None => write!(f, "{{}}"),
//     }
// }

// maybe this is overkill and we should just use fmt::Display an convert to string when creating Context?
#[derive(Debug, Clone)]
pub enum ContextValue {
    S(String),
    I(i64),
    F(f64),
}

impl fmt::Display for ContextValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContextValue::S(v) => write!(f, "{}", v),
            ContextValue::I(v) => write!(f, "{}", v),
            ContextValue::F(v) => write!(f, "{}", v),
        }
    }
}

impl From<String> for ContextValue {
    fn from(str: String) -> Self {
        Self::S(str)
    }
}

impl From<&str> for ContextValue {
    fn from(str: &str) -> Self {
        Self::S(str.to_string())
    }
}

impl From<i64> for ContextValue {
    fn from(int: i64) -> Self {
        Self::I(int)
    }
}

impl From<usize> for ContextValue {
    fn from(u: usize) -> Self {
        Self::I(u as i64)
    }
}

impl From<f64> for ContextValue {
    fn from(f: f64) -> Self {
        Self::F(f)
    }
}

impl ToPyObject for ContextValue {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            ContextValue::S(v) => v.into_py(py),
            ContextValue::I(v) => v.into_py(py),
            ContextValue::F(v) => v.into_py(py),
        }
    }
}
