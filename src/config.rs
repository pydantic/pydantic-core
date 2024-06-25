use pyo3::prelude::*;
use pyo3::types::PyType;
use pyo3::{exceptions::PyValueError, types::PyDict, FromPyObject, IntoPy, PyAny, PyObject, PyResult, Python};

#[derive(Debug, Clone, Default)]
pub struct CoreConfig {
    pub title: Option<String>,
    pub strict: Option<bool>,
    pub extra_fields_behavior: Option<ExtraBehavior>,
    pub typed_dict_total: Option<bool>,
    pub from_attributes: Option<bool>,
    pub loc_by_alias: Option<bool>,
    pub revalidate_instances: Option<RevalidateInstances>,
    pub validate_default: Option<bool>,
    pub populate_by_name: Option<bool>,
    pub str_max_length: Option<i32>,
    pub str_min_length: Option<i32>,
    pub str_strip_whitespace: Option<bool>,
    pub str_to_lower: Option<bool>,
    pub str_to_upper: Option<bool>,
    pub allow_inf_nan: Option<bool>,
    pub ser_json_timedelta: Option<SerJsonTimedelta>,
    pub ser_json_bytes: Option<SerJsonBytes>,
    pub ser_json_inf_nan: Option<SerJsonInfNan>,
    pub hide_input_in_errors: Option<bool>,
    pub validation_error_cause: Option<bool>,
    pub coerce_numbers_to_str: Option<bool>,
    pub regex_engine: Option<RegexEngine>,
    pub cache_strings: Option<CacheStrings>,
}

impl TryFrom<Bound<'_, PyDict>> for CoreConfig {
    type Error = PyErr;
    fn try_from(value: Bound<'_, PyDict>) -> Result<Self, Self::Error> {
        Ok(CoreConfig {
            title: value.get_item("title")?.map(|v| v.extract()).transpose()?,
            strict: value.get_item("strict")?.map(|v| v.extract()).transpose()?,
            extra_fields_behavior: value
                .get_item("extra_fields_behavior")?
                .map(|v| v.extract())
                .transpose()?,
            typed_dict_total: value.get_item("typed_dict_total")?.map(|v| v.extract()).transpose()?,
            from_attributes: value.get_item("from_attributes")?.map(|v| v.extract()).transpose()?,
            loc_by_alias: value.get_item("loc_by_alias")?.map(|v| v.extract()).transpose()?,
            revalidate_instances: value
                .get_item("revalidate_instances")?
                .map(|v| v.extract())
                .transpose()?,
            validate_default: value.get_item("validate_default")?.map(|v| v.extract()).transpose()?,
            populate_by_name: value.get_item("populate_by_name")?.map(|v| v.extract()).transpose()?,
            str_max_length: value.get_item("str_max_length")?.map(|v| v.extract()).transpose()?,
            str_min_length: value.get_item("str_min_length")?.map(|v| v.extract()).transpose()?,
            str_strip_whitespace: value
                .get_item("str_strip_whitespace")?
                .map(|v| v.extract())
                .transpose()?,
            str_to_lower: value.get_item("str_to_lower")?.map(|v| v.extract()).transpose()?,
            str_to_upper: value.get_item("str_to_upper")?.map(|v| v.extract()).transpose()?,
            allow_inf_nan: value.get_item("allow_inf_nan")?.map(|v| v.extract()).transpose()?,
            ser_json_timedelta: value.get_item("ser_json_timedelta")?.map(|v| v.extract()).transpose()?,
            ser_json_bytes: value.get_item("ser_json_bytes")?.map(|v| v.extract()).transpose()?,
            ser_json_inf_nan: value.get_item("ser_json_inf_nan")?.map(|v| v.extract()).transpose()?,
            hide_input_in_errors: value
                .get_item("hide_input_in_errors")?
                .map(|v| v.extract())
                .transpose()?,
            validation_error_cause: value
                .get_item("validation_error_cause")?
                .map(|v| v.extract())
                .transpose()?,
            coerce_numbers_to_str: value
                .get_item("coerce_numbers_to_str")?
                .map(|v| v.extract())
                .transpose()?,
            regex_engine: value.get_item("regex_engine")?.map(|v| v.extract()).transpose()?,
            cache_strings: value.get_item("cache_strings")?.map(|v| v.extract()).transpose()?,
        })
    }
}

impl IntoPy<PyObject> for CoreConfig {
    fn into_py(self, py: Python<'_>) -> PyObject {
        let dict = PyDict::new_bound(py);
        if let Some(title) = self.title {
            dict.set_item("title", title).unwrap();
        };
        if let Some(strict) = self.strict {
            dict.set_item("strict", strict).unwrap();
        };
        if let Some(extra_fields_behavior) = self.extra_fields_behavior {
            let value = match extra_fields_behavior {
                ExtraBehavior::Allow => "allow",
                ExtraBehavior::Ignore => "ignore",
                ExtraBehavior::Error => "error",
            };
            dict.set_item("extra_fields_behavior", value).unwrap();
        };
        if let Some(typed_dict_total) = self.typed_dict_total {
            dict.set_item("typed_dict_total", typed_dict_total).unwrap();
        };
        if let Some(from_attributes) = self.from_attributes {
            dict.set_item("from_attributes", from_attributes).unwrap();
        };
        if let Some(loc_by_alias) = self.loc_by_alias {
            dict.set_item("loc_by_alias", loc_by_alias).unwrap();
        };
        if let Some(revalidate_instances) = self.revalidate_instances {
            let value = match revalidate_instances {
                RevalidateInstances::Always => "always",
                RevalidateInstances::Never => "never",
                RevalidateInstances::SubclassInstances => "subclass_instances",
            };
            dict.set_item("revalidate_instances", value).unwrap();
        };
        if let Some(validate_default) = self.validate_default {
            dict.set_item("validate_default", validate_default).unwrap();
        };
        if let Some(populate_by_name) = self.populate_by_name {
            dict.set_item("populate_by_name", populate_by_name).unwrap();
        };
        if let Some(str_max_length) = self.str_max_length {
            dict.set_item("str_max_length", str_max_length).unwrap();
        };
        if let Some(str_min_length) = self.str_min_length {
            dict.set_item("str_min_length", str_min_length).unwrap();
        };
        if let Some(str_strip_whitespace) = self.str_strip_whitespace {
            dict.set_item("str_strip_whitespace", str_strip_whitespace).unwrap();
        };
        if let Some(str_to_lower) = self.str_to_lower {
            dict.set_item("str_to_lower", str_to_lower).unwrap();
        };
        if let Some(str_to_upper) = self.str_to_upper {
            dict.set_item("str_to_upper", str_to_upper).unwrap();
        };
        if let Some(allow_inf_nan) = self.allow_inf_nan {
            dict.set_item("allow_inf_nan", allow_inf_nan).unwrap();
        };
        if let Some(ser_json_timedelta) = self.ser_json_timedelta {
            let value = match ser_json_timedelta {
                SerJsonTimedelta::Iso8601 => "iso8601",
                SerJsonTimedelta::Float => "float",
            };
            dict.set_item("ser_json_timedelta", value).unwrap();
        };
        if let Some(ser_json_bytes) = self.ser_json_bytes {
            let value = match ser_json_bytes {
                SerJsonBytes::Utf8 => "utf8",
                SerJsonBytes::Base64 => "base64",
                SerJsonBytes::Hex => "hex",
            };
            dict.set_item("ser_json_bytes", value).unwrap();
        };
        if let Some(ser_json_inf_nan) = self.ser_json_inf_nan {
            let value = match ser_json_inf_nan {
                SerJsonInfNan::Null => "null",
                SerJsonInfNan::Constants => "constants",
                SerJsonInfNan::Strings => "strings",
            };
            dict.set_item("ser_json_inf_nan", value).unwrap();
        };
        if let Some(hide_input_in_errors) = self.hide_input_in_errors {
            dict.set_item("hide_input_in_errors", hide_input_in_errors).unwrap();
        };
        if let Some(validation_error_cause) = self.validation_error_cause {
            dict.set_item("validation_error_cause", validation_error_cause).unwrap();
        };
        if let Some(coerce_numbers_to_str) = self.coerce_numbers_to_str {
            dict.set_item("coerce_numbers_to_str", coerce_numbers_to_str).unwrap();
        };
        if let Some(regex_engine) = self.regex_engine {
            let value = match regex_engine {
                RegexEngine::RustRegex => "rust_regex",
                RegexEngine::PythonRe => "python_re",
            };
            dict.set_item("regex_engine", value).unwrap();
        };
        if let Some(cache_strings) = self.cache_strings {
            let value = match cache_strings {
                CacheStrings::All => "all",
                CacheStrings::Keys => "keys",
                CacheStrings::None => "none",
            };
            dict.set_item("cache_strings", value).unwrap();
        };
        dict.into()
    }
}

#[derive(Debug, Clone)]
pub enum ExtraBehavior {
    Allow,
    Ignore,
    Error,
}

impl FromPyObject<'_> for ExtraBehavior {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let value = ob.extract::<String>()?;
        match value.as_str() {
            "allow" => Ok(ExtraBehavior::Allow),
            "ignore" => Ok(ExtraBehavior::Ignore),
            "error" => Ok(ExtraBehavior::Error),
            _ => Err(PyValueError::new_err("Invalid value for extra_fields_behavior")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RevalidateInstances {
    Always,
    Never,
    SubclassInstances,
}

impl FromPyObject<'_> for RevalidateInstances {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let value = ob.extract::<String>()?;
        match value.as_str() {
            "always" => Ok(RevalidateInstances::Always),
            "never" => Ok(RevalidateInstances::Never),
            "subclass_instances" => Ok(RevalidateInstances::SubclassInstances),
            _ => Err(PyValueError::new_err("Invalid value for revalidate_instances")),
        }
    }
}

impl RevalidateInstances {
    pub fn should_revalidate(&self, input: &Bound<'_, PyAny>, class: &Bound<'_, PyType>) -> bool {
        match self {
            RevalidateInstances::Always => true,
            RevalidateInstances::Never => false,
            RevalidateInstances::SubclassInstances => !input.is_exact_instance(class),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SerJsonTimedelta {
    Iso8601,
    Float,
}

impl FromPyObject<'_> for SerJsonTimedelta {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let value = ob.extract::<String>()?;
        match value.as_str() {
            "iso8601" => Ok(SerJsonTimedelta::Iso8601),
            "float" => Ok(SerJsonTimedelta::Float),
            _ => Err(PyValueError::new_err("Invalid value for ser_json_timedelta")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SerJsonBytes {
    Utf8,
    Base64,
    Hex,
}

impl FromPyObject<'_> for SerJsonBytes {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let value = ob.extract::<String>()?;
        match value.as_str() {
            "utf8" => Ok(SerJsonBytes::Utf8),
            "base64" => Ok(SerJsonBytes::Base64),
            "hex" => Ok(SerJsonBytes::Hex),
            _ => Err(PyValueError::new_err("Invalid value for ser_json_bytes")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SerJsonInfNan {
    Null,
    Constants,
    Strings,
}

impl FromPyObject<'_> for SerJsonInfNan {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let value = ob.extract::<String>()?;
        match value.as_str() {
            "null" => Ok(SerJsonInfNan::Null),
            "constants" => Ok(SerJsonInfNan::Constants),
            "strings" => Ok(SerJsonInfNan::Strings),
            _ => Err(PyValueError::new_err("Invalid value for ser_json_inf_nan")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RegexEngine {
    RustRegex,
    PythonRe,
}

impl FromPyObject<'_> for RegexEngine {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let value = ob.extract::<String>()?;
        match value.as_str() {
            "rust_regex" => Ok(RegexEngine::RustRegex),
            "python_re" => Ok(RegexEngine::PythonRe),
            _ => Err(PyValueError::new_err("Invalid value for regex_engine")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CacheStrings {
    All,
    Keys,
    None,
}

impl FromPyObject<'_> for CacheStrings {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let value = ob.extract::<String>()?;
        match value.as_str() {
            "all" => Ok(CacheStrings::All),
            "keys" => Ok(CacheStrings::Keys),
            "none" => Ok(CacheStrings::None),
            _ => Err(PyValueError::new_err("Invalid value for cache_strings")),
        }
    }
}
