use ahash::AHashSet;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::build_tools::{py_err, SchemaDict};
use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::Input;
use crate::recursion_guard::RecursionGuard;

use super::literal::expected_repr_name;
use super::{BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct UrlValidator {
    host_required: bool,
    max_length: Option<usize>,
    allowed_schemes: Option<AHashSet<String>>,
    expected_repr: Option<String>,
    name: String,
}

impl BuildValidator for UrlValidator {
    const EXPECTED_TYPE: &'static str = "url";

    fn build(
        schema: &PyDict,
        _config: Option<&PyDict>,
        _build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let (allowed_schemes, expected_repr, name): (Option<AHashSet<String>>, Option<String>, String) =
            match schema.get_as::<&PyList>(intern!(schema.py(), "allowed_schemes"))? {
                Some(list) => {
                    if list.is_empty() {
                        return py_err!(r#""allowed_schemes" should have length > 0"#);
                    }

                    let mut expected: AHashSet<String> = AHashSet::new();
                    let mut repr_args = Vec::new();
                    for item in list.iter() {
                        let str = item.extract()?;
                        repr_args.push(format!("'{str}'"));
                        expected.insert(str);
                    }
                    let (repr, name) = expected_repr_name(repr_args, "literal");
                    (Some(expected), Some(repr), name)
                }
                None => (None, None, Self::EXPECTED_TYPE.to_string()),
            };

        Ok(Self {
            host_required: schema.get_as(intern!(schema.py(), "host_required"))?.unwrap_or(false),
            max_length: schema.get_as(intern!(schema.py(), "max_length"))?,
            allowed_schemes,
            expected_repr,
            name,
        }
        .into())
    }
}

impl Validator for UrlValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        _slots: &'data [CombinedValidator],
        _recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let either_str = input.validate_str(extra.strict.unwrap_or(false))?;

        let cow = either_str.as_cow()?;
        let str = cow.as_ref();

        if let Some(max_length) = self.max_length {
            if str.len() > max_length {
                return Err(ValError::new(ErrorType::UrlTooLong { max_length }, input));
            }
        }

        let lib_url = url::Url::parse(str)
            .map_err(move |e| ValError::new(ErrorType::UrlError { error: e.to_string() }, input))?;

        if let Some(ref allowed_schemes) = self.allowed_schemes {
            if !allowed_schemes.contains(lib_url.scheme()) {
                let expected_schemas = self.expected_repr.as_ref().unwrap().clone();
                return Err(ValError::new(ErrorType::UrlSchema { expected_schemas }, input));
            }
        }
        if self.host_required && !lib_url.has_host() {
            return Err(ValError::new(ErrorType::UrlHostRequired, input));
        }
        Ok(Url { lib_url }.into_py(py))
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[pyclass(module = "pydantic_core._pydantic_core")]
#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Url {
    lib_url: url::Url,
}

#[pymethods]
impl Url {
    #[getter]
    pub fn scheme(&self) -> &str {
        self.lib_url.scheme()
    }

    #[getter]
    pub fn username(&self) -> Option<&str> {
        match self.lib_url.username() {
            "" => None,
            user => Some(user),
        }
    }

    #[getter]
    pub fn password(&self) -> Option<&str> {
        self.lib_url.password()
    }

    #[getter]
    pub fn host(&self) -> Option<&str> {
        self.lib_url.host_str()
    }

    #[getter]
    pub fn host_type(&self) -> Option<&'static str> {
        match self.lib_url.host() {
            Some(url::Host::Domain(domain)) if domain.starts_with("xn--") => Some("international_domain"),
            Some(url::Host::Domain(_)) => Some("domain"),
            Some(url::Host::Ipv4(_)) => Some("ipv4"),
            Some(url::Host::Ipv6(_)) => Some("ipv6"),
            None => None,
        }
    }

    #[getter]
    pub fn port(&self) -> Option<u16> {
        self.lib_url.port()
    }

    #[getter]
    pub fn path(&self) -> Option<&str> {
        match self.lib_url.path() {
            "" => None,
            path => Some(path),
        }
    }

    #[getter]
    pub fn query(&self) -> Option<&str> {
        self.lib_url.query()
    }

    pub fn query_params(&self, py: Python) -> PyObject {
        // TODO remove `collect` when we have https://github.com/PyO3/pyo3/pull/2676
        self.lib_url
            .query_pairs()
            .map(|(key, value)| (key, value).into_py(py))
            .collect::<Vec<PyObject>>()
            .into_py(py)
    }

    #[getter]
    pub fn fragment(&self) -> Option<&str> {
        self.lib_url.fragment()
    }

    pub fn __str__(&self) -> String {
        self.lib_url.to_string()
    }

    pub fn __repr__(&self) -> String {
        format!("Url('{}')", self.lib_url)
    }
}
