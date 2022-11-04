use idna::punycode::decode_to_string;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use url::Url;

#[pyclass(name = "Url", module = "pydantic_core._pydantic_core")]
#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct PyUrl {
    lib_url: Url,
}

impl PyUrl {
    pub fn new(lib_url: Url) -> Self {
        Self { lib_url }
    }

    pub fn into_url(self) -> Url {
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

    // string representation of the host, with punycode decoded when appropriate
    pub fn unicode_host(&self) -> Option<String> {
        match self.lib_url.host() {
            Some(url::Host::Domain(domain)) if is_punnycode_domain(&self.lib_url, domain) => decode_punycode(domain),
            _ => self.lib_url.host_str().map(|h| h.to_string()),
        }
    }

    #[getter]
    pub fn host_type(&self) -> Option<&'static str> {
        match self.lib_url.host() {
            Some(url::Host::Domain(domain)) if is_punnycode_domain(&self.lib_url, domain) => Some("punycode_domain"),
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
        // `query_pairs` is a pure iterator, so can't implement `ExactSizeIterator`, hence we need the temporary `Vec`
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

    // string representation of the URL, with punycode decoded when appropriate
    pub fn unicode_string(&self) -> String {
        let s = self.lib_url.to_string();

        match self.lib_url.host() {
            Some(url::Host::Domain(domain)) if is_punnycode_domain(&self.lib_url, domain) => {
                // we know here that we have a punycode domain, so we simply replace the first instance
                // of the punycode domain with the decoded domain
                // this is ugly, but since `slice()`, `host_start` and `host_end` are all private to `Url`,
                // we have no better option, since the `schema` has to be `https`, `http` etc, (see `is_special` above),
                // we can safely assume that the first match for the domain, is the domain
                match decode_punycode(domain) {
                    Some(decoded) => s.replacen(domain, &decoded, 1),
                    None => s,
                }
            }
            _ => s,
        }
    }

    pub fn __str__(&self) -> &str {
        self.lib_url.as_str()
    }

    pub fn __repr__(&self) -> String {
        format!("Url('{}')", self.lib_url)
    }
}

#[pyclass(name = "MultiHostUrl", module = "pydantic_core._pydantic_core")]
#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct PyMultiHostUrl {
    ref_url: PyUrl,
    extra_urls: Option<Vec<Url>>,
}

impl PyMultiHostUrl {
    pub fn new(ref_url: Url, extra_urls: Option<Vec<Url>>) -> Self {
        Self {
            ref_url: PyUrl::new(ref_url),
            extra_urls,
        }
    }
}

#[pymethods]
impl PyMultiHostUrl {
    #[getter]
    pub fn scheme(&self) -> &str {
        self.ref_url.scheme()
    }

    pub fn hosts<'s, 'py>(&'s self, py: Python<'py>) -> PyResult<Vec<&'py PyDict>> {
        if let Some(extra_urls) = &self.extra_urls {
            let mut hosts = Vec::with_capacity(extra_urls.len() + 1);
            for url in extra_urls {
                hosts.push(host_to_dict(py, &url)?);
            }
            hosts.push(host_to_dict(py, &self.ref_url.lib_url)?);
            Ok(hosts)
        } else {
            Ok(vec![host_to_dict(py, &self.ref_url.lib_url)?])
        }
    }

    #[getter]
    pub fn path(&self) -> Option<&str> {
        self.ref_url.path()
    }

    #[getter]
    pub fn query(&self) -> Option<&str> {
        self.ref_url.query()
    }

    pub fn query_params(&self, py: Python) -> PyObject {
        self.ref_url.query_params(py)
    }

    #[getter]
    pub fn fragment(&self) -> Option<&str> {
        self.ref_url.fragment()
    }

    pub fn __str__(&self) -> String {
        if let Some(extra_urls) = &self.extra_urls {
            let schema = self.ref_url.lib_url.scheme();
            let host_offset = schema.len() + 3;

            let mut full_url = self.ref_url.lib_url.to_string();
            full_url.insert(host_offset, ',');

            // special urls will have had a trailing slash asked, non-special urls will not
            // hence we need to remove the last char if the schema is special
            let sub = if schema_is_special(schema) { 1 } else { 0 };

            // all extra urls have no path (except a `/`), so we can just have a slice up to -1
            let hosts = extra_urls
                .iter()
                .map(|url| {
                    let str = url.as_str();
                    &str[host_offset..str.len() - sub]
                })
                .collect::<Vec<&str>>()
                .join(",");
            full_url.insert_str(host_offset, &hosts);
            full_url
        } else {
            self.ref_url.__str__().to_string()
        }
    }

    pub fn __repr__(&self) -> String {
        format!("Url('{}')", self.__str__())
    }
}

fn host_to_dict<'a, 'b>(py: Python<'a>, lib_url: &'b Url) -> PyResult<&'a PyDict> {
    let dict = PyDict::new(py);
    dict.set_item(
        "username",
        match lib_url.username() {
            "" => py.None(),
            user => user.into_py(py),
        },
    )?;
    dict.set_item("password", lib_url.password())?;
    dict.set_item("host", lib_url.host_str())?;
    dict.set_item("port", lib_url.port())?;

    Ok(dict)
}

fn decode_punycode(domain: &str) -> Option<String> {
    let mut result = String::with_capacity(domain.len());
    for chunk in domain.split('.') {
        if let Some(stripped) = chunk.strip_prefix(PUNYCODE_PREFIX) {
            result.push_str(&decode_to_string(stripped)?);
        } else {
            result.push_str(chunk);
        }
        result.push('.');
    }
    result.pop();
    Some(result)
}

static PUNYCODE_PREFIX: &str = "xn--";

fn is_punnycode_domain(lib_url: &Url, domain: &str) -> bool {
    schema_is_special(lib_url.scheme()) && domain.split('.').any(|part| part.starts_with(PUNYCODE_PREFIX))
}

// based on https://github.com/servo/rust-url/blob/1c1e406874b3d2aa6f36c5d2f3a5c2ea74af9efb/url/src/parser.rs#L161-L167
pub fn schema_is_special(schema: &str) -> bool {
    matches!(schema, "http" | "https" | "ws" | "wss" | "ftp" | "file")
}
