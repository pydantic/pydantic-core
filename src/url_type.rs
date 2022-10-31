use pyo3::prelude::*;

#[pyclass(module = "pydantic_core._pydantic_core", subclass)]
#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Url {
    url: url::Url,
}

#[pymethods]
impl Url {
    #[new]
    pub fn py_new(raw_url: &str) -> PyResult<Self> {
        let url = url::Url::parse(raw_url).unwrap();
        Ok(Self { url })
    }

    #[getter]
    pub fn scheme(&self) -> &str {
        self.url.scheme()
    }

    #[getter]
    pub fn username(&self) -> Option<&str> {
        match self.url.username() {
            "" => None,
            user => Some(user),
        }
    }

    #[getter]
    pub fn password(&self) -> Option<&str> {
        self.url.password()
    }

    #[getter]
    pub fn host(&self) -> Option<&str> {
        self.url.host_str()
    }

    #[getter]
    pub fn host_type(&self) -> Option<&'static str> {
        match self.url.host() {
            Some(url::Host::Domain(domain)) if domain.starts_with("xn--") => Some("international_domain"),
            Some(url::Host::Domain(_)) => Some("domain"),
            Some(url::Host::Ipv4(_)) => Some("ipv4"),
            Some(url::Host::Ipv6(_)) => Some("ipv6"),
            None => None,
        }
    }

    #[getter]
    pub fn port(&self) -> Option<u16> {
        self.url.port()
    }

    #[getter]
    pub fn path(&self) -> &str {
        self.url.path()
    }

    #[getter]
    pub fn query(&self) -> Option<&str> {
        self.url.query()
    }

    pub fn query_params(&self, py: Python) -> PyObject {
        // TODO remove `collect` when we have https://github.com/PyO3/pyo3/pull/2676
        self.url
            .query_pairs()
            .map(|(key, value)| (key, value).into_py(py))
            .collect::<Vec<PyObject>>()
            .into_py(py)
    }

    #[getter]
    pub fn fragment(&self) -> Option<&str> {
        self.url.fragment()
    }

    pub fn __str__(&self) -> String {
        self.url.to_string()
    }

    pub fn __repr__(&self) -> String {
        format!("Url('{}')", self.url)
    }
}
