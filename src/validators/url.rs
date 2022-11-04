use std::cell::RefCell;
use std::iter::Peekable;
use std::str::Chars;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use ahash::AHashSet;
use url::{ParseError, SyntaxViolation, Url};

use crate::build_tools::{is_strict, py_err, SchemaDict};
use crate::errors::{ErrorType, ValError, ValResult};
use crate::input::Input;
use crate::recursion_guard::RecursionGuard;
use crate::url::{schema_is_special, PyMultiHostUrl, PyUrl};

use super::literal::expected_repr_name;
use super::{BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

type AllowedSchemas = Option<(AHashSet<String>, String)>;

#[derive(Debug, Clone)]
pub struct UrlValidator {
    strict: bool,
    host_required: bool,
    max_length: Option<usize>,
    allowed_schemes: AllowedSchemas,
    name: String,
}

impl BuildValidator for UrlValidator {
    const EXPECTED_TYPE: &'static str = "url";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        _build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let (allowed_schemes, name) = get_allowed_schemas(schema, Self::EXPECTED_TYPE)?;

        Ok(Self {
            strict: is_strict(schema, config)?,
            host_required: schema.get_as(intern!(schema.py(), "host_required"))?.unwrap_or(false),
            max_length: schema.get_as(intern!(schema.py(), "max_length"))?,
            allowed_schemes,
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
        let lib_url = self.get_url(input, extra.strict.unwrap_or(self.strict))?;

        if let Some((ref allowed_schemes, ref expected_schemas_repr)) = self.allowed_schemes {
            if !allowed_schemes.contains(lib_url.scheme()) {
                let expected_schemas = expected_schemas_repr.clone();
                return Err(ValError::new(ErrorType::UrlSchema { expected_schemas }, input));
            }
        }
        if self.host_required && !lib_url.has_host() {
            return Err(ValError::new(ErrorType::UrlHostRequired, input));
        }
        Ok(PyUrl::new(lib_url).into_py(py))
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

impl UrlValidator {
    fn get_url<'s, 'data>(&'s self, input: &'data impl Input<'data>, strict: bool) -> ValResult<'data, Url> {
        match input.validate_str(strict) {
            Ok(either_str) => {
                let cow = either_str.as_cow()?;
                let url_str = cow.as_ref();

                self.check_length(input, url_str)?;

                parse_url(url_str, input, strict)
            }
            Err(_) => {
                // we don't need to worry about whether the url was parsed in strict mode before,
                // even if it was, any syntax errors would have been fixed by the first validation
                if let Some(py_url) = input.input_as_url() {
                    let lib_url = py_url.into_url();
                    self.check_length(input, lib_url.as_str())?;
                    Ok(lib_url)
                } else if let Some(multi_host_url) = input.input_as_multi_host_url() {
                    let url_str = multi_host_url.__str__();
                    self.check_length(input, &url_str)?;

                    parse_url(&url_str, input, strict)
                } else {
                    Err(ValError::new(ErrorType::UrlType, input))
                }
            }
        }
    }

    fn check_length<'s, 'data>(&self, input: &'data impl Input<'data>, url_str: &str) -> ValResult<'data, ()> {
        if let Some(max_length) = self.max_length {
            if url_str.len() > max_length {
                return Err(ValError::new(ErrorType::UrlTooLong { max_length }, input));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MultiHostUrlValidator {
    strict: bool,
    max_length: Option<usize>,
    allowed_schemes: AllowedSchemas,
    name: String,
}

impl BuildValidator for MultiHostUrlValidator {
    const EXPECTED_TYPE: &'static str = "multi-host-url";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        _build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let (allowed_schemes, name) = get_allowed_schemas(schema, Self::EXPECTED_TYPE)?;

        Ok(Self {
            strict: is_strict(schema, config)?,
            max_length: schema.get_as(intern!(schema.py(), "max_length"))?,
            allowed_schemes,
            name,
        }
        .into())
    }
}

impl Validator for MultiHostUrlValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        _slots: &'data [CombinedValidator],
        _recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let multi_url = self.get_url(input, extra.strict.unwrap_or(self.strict))?;

        if let Some((ref allowed_schemes, ref expected_schemas_repr)) = self.allowed_schemes {
            if !allowed_schemes.contains(multi_url.scheme()) {
                let expected_schemas = expected_schemas_repr.clone();
                return Err(ValError::new(ErrorType::UrlSchema { expected_schemas }, input));
            }
        }
        Ok(multi_url.into_py(py))
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

impl MultiHostUrlValidator {
    fn get_url<'s, 'data>(&'s self, input: &'data impl Input<'data>, strict: bool) -> ValResult<'data, PyMultiHostUrl> {
        match input.validate_str(strict) {
            Ok(either_str) => {
                let cow = either_str.as_cow()?;
                let url_str = cow.as_ref();

                self.check_length(input, || url_str.len())?;

                parse_multihost_url(url_str, input, strict)
            }
            Err(_) => {
                // we don't need to worry about whether the url was parsed in strict mode before,
                // even if it was, any syntax errors would have been fixed by the first validation
                if let Some(multi_url) = input.input_as_multi_host_url() {
                    self.check_length(input, || multi_url.__str__().len())?;
                    Ok(multi_url)
                } else if let Some(py_url) = input.input_as_url() {
                    let lib_url = py_url.into_url();
                    self.check_length(input, || lib_url.as_str().len())?;
                    Ok(PyMultiHostUrl::new(lib_url, None))
                } else {
                    Err(ValError::new(ErrorType::UrlType, input))
                }
            }
        }
    }

    fn check_length<'s, 'data, F>(&self, input: &'data impl Input<'data>, func: F) -> ValResult<'data, ()>
    where
        F: FnOnce() -> usize,
    {
        if let Some(max_length) = self.max_length {
            if func() > max_length {
                return Err(ValError::new(ErrorType::UrlTooLong { max_length }, input));
            }
        }
        Ok(())
    }
}

fn parse_multihost_url<'url, 'input>(
    url_str: &'url str,
    input: &'input impl Input<'input>,
    strict: bool,
) -> ValResult<'input, PyMultiHostUrl> {
    let mut chars = PositionedPeekable::new(url_str);
    // TODO consume whitespace

    macro_rules! parsing_err {
        ($parse_error:expr) => {
            Err(ValError::new(
                ErrorType::UrlParsing {
                    error: $parse_error.to_string(),
                },
                input,
            ))
        };
    }

    // consume the url schema, taken from `parse_scheme`
    // https://github.com/servo/rust-url/blob/v2.3.1/url/src/parser.rs#L387-L411
    while let Some(c) = chars.next() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '+' | '-' | '.' => (),
            ':' => break,
            _ => return parsing_err!(ParseError::RelativeUrlWithoutBase),
        }
    }
    let schema = &url_str[..chars.index - 1];

    // consume the double slash, or any number of slashes, including backslashes, taken from `parse_with_scheme`
    // https://github.com/servo/rust-url/blob/v2.3.1/url/src/parser.rs#L413-L456
    loop {
        let peek = chars.peek();
        if peek == Some(&'/') || peek == Some(&'\\') {
            chars.next();
        } else {
            break;
        }
    }

    // process host and port, splitting based on `,`, some logic taken from `parse_host`
    // https://github.com/servo/rust-url/blob/v2.3.1/url/src/parser.rs#L971-L1026
    let mut hosts: Vec<&str> = Vec::with_capacity(3);
    let mut start = chars.index;
    while let Some(c) = chars.next() {
        match c {
            '\\' if schema_is_special(schema) => break,
            '/' | '?' | '#' => break,
            ',' => {
                let end = chars.index - 1;
                if start == end {
                    return parsing_err!(ParseError::EmptyHost);
                }
                hosts.push(&url_str[start..end]);
                start = chars.index;
            }
            _ => (),
        }
    }

    let end = chars.index - 1;
    if start == end {
        return parsing_err!(ParseError::EmptyHost);
    }
    let rest = &url_str[chars.index - 1..];
    let reconstructed_url = format!("{schema}://{}{rest}", &url_str[start..end]);
    let ref_url = parse_url(&reconstructed_url, input, strict)?;
    if !ref_url.has_host() {
        return parsing_err!(ParseError::EmptyHost);
    }

    if hosts.is_empty() {
        // just one host, for consistent behaviour, we parse the URL the same as multiple below
        Ok(PyMultiHostUrl::new(ref_url, None))
    } else {
        let extra_urls: Vec<Url> = hosts
            .iter()
            .map(|host| {
                let reconstructed_url = format!("{schema}://{host}");
                parse_url(&reconstructed_url, input, strict)
            })
            .collect::<ValResult<_>>()?;

        if extra_urls.iter().any(|url| !url.has_host()) {
            return parsing_err!(ParseError::EmptyHost);
        }

        Ok(PyMultiHostUrl::new(ref_url, Some(extra_urls)))
    }
}

fn parse_url<'url, 'input>(
    url_str: &'url str,
    input: &'input impl Input<'input>,
    strict: bool,
) -> ValResult<'input, Url> {
    // if we're in strict mode, we collect consider a syntax violation as an error
    if strict {
        // we could build a vec of syntax violations and return them all, but that seems like overkill
        // and unlike other parser style validators
        let vios: RefCell<Option<SyntaxViolation>> = RefCell::new(None);
        let r = Url::options()
            .syntax_violation_callback(Some(&|v| *vios.borrow_mut() = Some(v)))
            .parse(url_str);

        match r {
            Ok(url) => {
                if let Some(vio) = vios.into_inner() {
                    Err(ValError::new(
                        ErrorType::UrlSyntaxViolation {
                            error: vio.description().into(),
                        },
                        input,
                    ))
                } else {
                    Ok(url)
                }
            }
            Err(e) => Err(ValError::new(ErrorType::UrlParsing { error: e.to_string() }, input)),
        }
    } else {
        Url::parse(url_str).map_err(move |e| ValError::new(ErrorType::UrlParsing { error: e.to_string() }, input))
    }
}

fn get_allowed_schemas(schema: &PyDict, name: &'static str) -> PyResult<(AllowedSchemas, String)> {
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
            let (repr, name) = expected_repr_name(repr_args, name);
            Ok((Some((expected, repr)), name))
        }
        None => Ok((None, name.to_string())),
    }
}

struct PositionedPeekable<'a> {
    peekable: Peekable<Chars<'a>>,
    index: usize,
}

impl<'a> PositionedPeekable<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            peekable: input.chars().peekable(),
            index: 0,
        }
    }

    fn next(&mut self) -> Option<char> {
        self.index += 1;
        self.peekable.next()
    }

    fn peek(&mut self) -> Option<&char> {
        self.peekable.peek()
    }
}
