use pyo3::prelude::*;

#[pyclass(name = "Url", module = "pydantic_core._pydantic_core")]
#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct PyUrl {
    lib_url: url::Url,
}

impl PyUrl {
    pub fn new(lib_url: url::Url) -> Self {
        Self { lib_url }
    }

    pub fn into_url(self) -> url::Url {
        self.lib_url
    }
}

#[pymethods]
impl PyUrl {
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
