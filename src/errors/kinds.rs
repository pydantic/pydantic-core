use pyo3::prelude::*;
use pyo3::types::PyDict;

use strum::{Display, EnumMessage};

#[derive(Debug, Display, EnumMessage, Clone)]
#[strum(serialize_all = "snake_case")]
pub enum ErrorKind {
    #[strum(message = "Invalid input")]
    InvalidInput,
    #[strum(message = "Invalid JSON: {parser_error}")]
    InvalidJson { parser_error: String },
    // ---------------------
    // recursion error
    #[strum(message = "Recursion error - cyclic reference detected")]
    RecursionLoop,
    // ---------------------
    // typed dict specific errors
    #[strum(message = "Value must be a valid dictionary or instance to extract fields from")]
    DictAttributesType,
    #[strum(message = "Field required")]
    Missing,
    #[strum(message = "Extra values are not permitted")]
    ExtraForbidden,
    #[strum(message = "Model keys must be strings")]
    InvalidKey,
    #[strum(message = "Error extracting attribute: {error}")]
    GetAttributeError { error: String },
    // ---------------------
    // model class specific errors
    #[strum(message = "Value must be an instance of {class_name}")]
    ModelClassType { class_name: String },
    // ---------------------
    // None errors
    #[strum(message = "Value must be None/null")]
    NoneRequired,
    // boolean errors
    #[strum(message = "Value must be a valid boolean")]
    Bool,
    // ---------------------
    // generic comparison errors - used for all inequality comparisons
    #[strum(message = "Value must be greater than {gt}")]
    GreaterThan { gt: String },
    #[strum(message = "Value must be greater than or equal to {ge}")]
    GreaterThanEqual { ge: String },
    #[strum(message = "Value must be less than {lt}")]
    LessThan { lt: String },
    #[strum(message = "Value must be less than or equal to {le}")]
    LessThanEqual { le: String },
    // ---------------------
    // generic length errors - used for everything with a length except strings and bytes which need custom messages
    #[strum(message = "Input must have at least {min_length} items")]
    TooShort { min_length: usize },
    #[strum(message = "Input must have at most {max_length} items")]
    TooLong { max_length: usize },
    // ---------------------
    // string errors
    #[strum(message = "Value must be a valid string")]
    StrType,
    #[strum(message = "Value must be a valid string, unable to parse raw data as a unicode string")]
    StrUnicode,
    #[strum(
        message = "String must have at least {min_length} characters",
        serialize = "too_short"
    )]
    StrTooShort { min_length: usize },
    #[strum(message = "String must have at most {max_length} characters", serialize = "too_long")]
    StrTooLong { max_length: usize },
    #[strum(message = "String must match pattern '{pattern}'")]
    StrPatternMismatch { pattern: String },
    // ---------------------
    // dict errors
    #[strum(message = "Value must be a valid dictionary")]
    DictType,
    #[strum(message = "Unable to convert mapping to a dictionary, error: {error}")]
    DictFromMapping { error: String },
    // ---------------------
    // list errors
    #[strum(message = "Value must be a valid list/array")]
    ListType,
    // ---------------------
    // tuple errors
    #[strum(message = "Value must be a valid tuple")]
    TupleType,
    #[strum(message = "Tuple must have exactly {expected_length} item{plural}")]
    TupleLengthMismatch { expected_length: usize, plural: bool },
    // ---------------------
    // set errors
    #[strum(message = "Value must be a valid set")]
    SetType,
    // ---------------------
    // bool errors
    #[strum(message = "Value must be a valid boolean")]
    BoolType,
    #[strum(message = "Value must be a valid boolean, unable to interpret input")]
    BoolParsing,
    // ---------------------
    // int errors
    #[strum(message = "Value must be a valid integer")]
    IntType,
    #[strum(message = "Value must be a valid integer, unable to parse string as an integer")]
    IntParsing,
    #[strum(message = "Value must be a valid integer, got a number with a fractional part")]
    IntFromFloat,
    #[strum(message = "Value must be a valid integer, got {nan_value}")]
    IntNan { nan_value: &'static str },
    #[strum(serialize = "multiple_of", message = "Value must be a multiple of {multiple_of}")]
    IntMultipleOf { multiple_of: i64 },
    #[strum(serialize = "greater_than", message = "Value must be greater than {gt}")]
    IntGreaterThan { gt: i64 },
    #[strum(
        serialize = "greater_than_equal",
        message = "Value must be greater than or equal to {ge}"
    )]
    IntGreaterThanEqual { ge: i64 },
    #[strum(serialize = "less_than", message = "Value must be less than {lt}")]
    IntLessThan { lt: i64 },
    #[strum(serialize = "less_than_equal", message = "Value must be less than or equal to {le}")]
    IntLessThanEqual { le: i64 },
    // ---------------------
    // float errors
    #[strum(message = "Value must be a valid number")]
    FloatType,
    #[strum(message = "Value must be a valid number, unable to parse string as an number")]
    FloatParsing,
    #[strum(serialize = "multiple_of", message = "Value must be a multiple of {multiple_of}")]
    FloatMultipleOf { multiple_of: f64 },
    #[strum(serialize = "greater_than", message = "Value must be greater than {gt}")]
    FloatGreaterThan { gt: f64 },
    #[strum(
        serialize = "greater_than_equal",
        message = "Value must be greater than or equal to {ge}"
    )]
    FloatGreaterThanEqual { ge: f64 },
    #[strum(serialize = "less_than", message = "Value must be less than {lt}")]
    FloatLessThan { lt: f64 },
    #[strum(serialize = "less_than_equal", message = "Value must be less than or equal to {le}")]
    FloatLessThanEqual { le: f64 },
    // ---------------------
    // bytes errors
    #[strum(message = "Value must be a valid bytes")]
    BytesType,
    #[strum(message = "Data must have at least {min_length} bytes", serialize = "too_short")]
    BytesTooShort { min_length: usize },
    #[strum(message = "Data must have at most {max_length} bytes", serialize = "too_long")]
    BytesTooLong { max_length: usize },
    // ---------------------
    // python errors from functions (the messages here will not be used as we sett message in these cases)
    #[strum(message = "Invalid value: {error}")]
    ValueError { error: String },
    #[strum(message = "Assertion failed: {error}")]
    AssertionError { error: String },
    // ---------------------
    // literals
    #[strum(serialize = "literal_error", message = "Value must be {expected}")]
    LiteralSingleError { expected: String },
    #[strum(serialize = "literal_error", message = "Value must be one of: {expected}")]
    LiteralMultipleError { expected: String },
    // ---------------------
    // date errors
    #[strum(message = "Value must be a valid date")]
    DateType,
    #[strum(message = "Value must be a valid date in the format YYYY-MM-DD, {parsing_error}")]
    DateParsing { parsing_error: String },
    #[strum(message = "Value must be a valid date or datetime, {parsing_error}")]
    DateFromDatetimeParsing { parsing_error: String },
    #[strum(message = "Datetimes provided to dates must have zero time - e.g. be exact dates")]
    DateFromDatetimeInexact,
    // ---------------------
    // date errors
    #[strum(message = "Value must be a valid time")]
    TimeType,
    #[strum(message = "Value must be in a valid time format, {parsing_error}")]
    TimeParsing { parsing_error: String },
    // ---------------------
    // datetime errors
    #[strum(serialize = "datetime_type", message = "Value must be a valid datetime")]
    DateTimeType,
    #[strum(
        serialize = "datetime_parsing",
        message = "Value must be a valid datetime, {parsing_error}"
    )]
    DateTimeParsing { parsing_error: String },
    #[strum(
        serialize = "datetime_object_invalid",
        message = "Invalid datetime object, got {processing_error}"
    )]
    DateTimeObjectInvalid { processing_error: String },
    // ---------------------
    // frozenset errors
    #[strum(message = "Value must be a valid frozenset")]
    FrozenSetType,
}

impl Default for ErrorKind {
    fn default() -> Self {
        ErrorKind::InvalidInput
    }
}

fn create_py_dict<'a, I: IntoIterator<Item = (&'a str, PyObject)>>(
    py: Python<'a>,
    members: I,
) -> PyResult<Option<PyObject>> {
    let dict = PyDict::new(py);
    for (key, value) in members {
        dict.set_item(key, value)?;
    }
    Ok(Some(dict.into_py(py)))
}

macro_rules! py_dict {
    ($py:ident, $($value:expr),* $(,)?) => {{
        create_py_dict($py, [$((stringify!($value), $value.into_py($py)),)*])
    }};
}

impl ErrorKind {
    pub fn render(&self) -> String {
        let template = self.get_message().unwrap().to_string();
        match self {
            Self::InvalidJson { parser_error } => template.replace("{parser_error}", parser_error),
            Self::GetAttributeError { error } => template.replace("{error}", error),
            Self::ModelClassType { class_name } => template.replace("{class_name}", class_name),
            Self::GreaterThan { gt } => template.replace("{gt}", gt),
            Self::GreaterThanEqual { ge } => template.replace("{ge}", ge),
            Self::LessThan { lt } => template.replace("{lt}", lt),
            Self::LessThanEqual { le } => template.replace("{le}", le),
            Self::TooShort { min_length } => template.replace("{min_length}", &min_length.to_string()),
            Self::TooLong { max_length } => template.replace("{max_length}", &max_length.to_string()),
            Self::StrTooShort { min_length } => template.replace("{min_length}", &min_length.to_string()),
            Self::StrTooLong { max_length } => template.replace("{max_length}", &max_length.to_string()),
            Self::StrPatternMismatch { pattern } => template.replace("{pattern}", pattern),
            Self::DictFromMapping { error } => template.replace("{error}", error),
            Self::TupleLengthMismatch {
                expected_length,
                plural,
            } => template
                .replace("{expected_length}", &expected_length.to_string())
                .replace("{plural}", if *plural { "s" } else { "" }),
            Self::IntNan { nan_value } => template.replace("{nan_value}", nan_value),
            Self::IntMultipleOf { multiple_of } => template.replace("{multiple_of}", &multiple_of.to_string()),
            Self::IntGreaterThan { gt } => template.replace("{gt}", &gt.to_string()),
            Self::IntGreaterThanEqual { ge } => template.replace("{ge}", &ge.to_string()),
            Self::IntLessThan { lt } => template.replace("{lt}", &lt.to_string()),
            Self::IntLessThanEqual { le } => template.replace("{le}", &le.to_string()),
            Self::FloatMultipleOf { multiple_of } => template.replace("{multiple_of}", &multiple_of.to_string()),
            Self::FloatGreaterThan { gt } => template.replace("{gt}", &gt.to_string()),
            Self::FloatGreaterThanEqual { ge } => template.replace("{ge}", &ge.to_string()),
            Self::FloatLessThan { lt } => template.replace("{lt}", &lt.to_string()),
            Self::FloatLessThanEqual { le } => template.replace("{le}", &le.to_string()),
            Self::BytesTooShort { min_length } => template.replace("{min_length}", &min_length.to_string()),
            Self::BytesTooLong { max_length } => template.replace("{max_length}", &max_length.to_string()),
            Self::ValueError { error } => template.replace("{error}", error),
            Self::AssertionError { error } => template.replace("{error}", error),
            Self::LiteralSingleError { expected } => template.replace("{expected}", expected),
            Self::LiteralMultipleError { expected } => template.replace("{expected}", expected),
            Self::DateParsing { parsing_error } => template.replace("{parsing_error}", parsing_error),
            Self::DateFromDatetimeParsing { parsing_error } => template.replace("{parsing_error}", parsing_error),
            Self::TimeParsing { parsing_error } => template.replace("{parsing_error}", parsing_error),
            Self::DateTimeParsing { parsing_error } => template.replace("{parsing_error}", parsing_error),
            Self::DateTimeObjectInvalid { processing_error } => {
                template.replace("{processing_error}", processing_error)
            }
            _ => template,
        }
    }

    pub fn py_dict(&self, py: Python) -> PyResult<Option<PyObject>> {
        match self {
            Self::InvalidJson { parser_error } => py_dict!(py, parser_error),
            Self::GetAttributeError { error } => py_dict!(py, error),
            Self::ModelClassType { class_name } => py_dict!(py, class_name),
            Self::GreaterThan { gt } => py_dict!(py, gt),
            Self::GreaterThanEqual { ge } => py_dict!(py, ge),
            Self::LessThan { lt } => py_dict!(py, lt),
            Self::LessThanEqual { le } => py_dict!(py, le),
            Self::TooShort { min_length } => py_dict!(py, min_length),
            Self::TooLong { max_length } => py_dict!(py, max_length),
            Self::StrTooShort { min_length } => py_dict!(py, min_length),
            Self::StrTooLong { max_length } => py_dict!(py, max_length),
            Self::StrPatternMismatch { pattern } => py_dict!(py, pattern),
            Self::DictFromMapping { error } => py_dict!(py, error),
            Self::TupleLengthMismatch {
                expected_length,
                plural,
            } => py_dict!(py, expected_length, plural),
            Self::IntNan { nan_value } => py_dict!(py, nan_value),
            Self::IntMultipleOf { multiple_of } => py_dict!(py, multiple_of),
            Self::IntGreaterThan { gt } => py_dict!(py, gt),
            Self::IntGreaterThanEqual { ge } => py_dict!(py, ge),
            Self::IntLessThan { lt } => py_dict!(py, lt),
            Self::IntLessThanEqual { le } => py_dict!(py, le),
            Self::FloatMultipleOf { multiple_of } => py_dict!(py, multiple_of),
            Self::FloatGreaterThan { gt } => py_dict!(py, gt),
            Self::FloatGreaterThanEqual { ge } => py_dict!(py, ge),
            Self::FloatLessThan { lt } => py_dict!(py, lt),
            Self::FloatLessThanEqual { le } => py_dict!(py, le),
            Self::BytesTooShort { min_length } => py_dict!(py, min_length),
            Self::BytesTooLong { max_length } => py_dict!(py, max_length),
            Self::ValueError { error } => py_dict!(py, error),
            Self::AssertionError { error } => py_dict!(py, error),
            Self::LiteralSingleError { expected } => py_dict!(py, expected),
            Self::LiteralMultipleError { expected } => py_dict!(py, expected),
            Self::DateParsing { parsing_error } => py_dict!(py, parsing_error),
            Self::DateFromDatetimeParsing { parsing_error } => py_dict!(py, parsing_error),
            Self::TimeParsing { parsing_error } => py_dict!(py, parsing_error),
            Self::DateTimeParsing { parsing_error } => py_dict!(py, parsing_error),
            Self::DateTimeObjectInvalid { processing_error } => py_dict!(py, processing_error),
            _ => Ok(None),
        }
    }
}
