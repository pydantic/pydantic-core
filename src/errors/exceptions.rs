use std::collections::HashMap;
use std::fmt;

use pyo3::exceptions::{PyException, PyTypeError, PyNotImplementedError};
use pyo3::pyclass::CompareOp;
use pyo3::types::{PyDict, PyString};
use pyo3::{intern, prelude::*, FromPyObject, Py};
use serde::Serialize;
use enum_dispatch::enum_dispatch;
use pythonize::{pythonize};
use strum::Display;


#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ContextValue {
    Float(f64),
    Boolean(bool),
    Int(i64),
    String(String),
    None(Option<i32>), // the inner type is ignored
}

impl FromPyObject<'_> for ContextValue {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        if let Ok(int) = obj.extract::<i64>() {
            Ok(Self::Int(int))
        } else if let Ok(float) = obj.extract::<f64>() {
            Ok(Self::Float(float))
        } else if let Ok(string) = obj.extract::<String>() {
            Ok(Self::String(string))
        } else if obj.is_none() {
            Ok(Self::None(None))
        } else {
            Err(
                PyTypeError::new_err(
                    format!(
                        "Expected int, float, string, bool or None, got {}", obj.get_type()
                    )
                )
            )
        }
    }
}

impl From<i64> for ContextValue {
    fn from(i: i64) -> Self {
        Self::Int(i)
    }
}

impl From<f64> for ContextValue {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl From<String> for ContextValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl fmt::Display for ContextValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Float(s) => write!(f, "{s}"),
            Self::Int(i) => write!(f, "{i}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Boolean(s) =>  write!(f, "{s}"),
            Self::None(_) =>  write!(f, "None"),
        }
    }
}

impl IntoPy<PyObject> for ContextValue {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            Self::Int(i) => i.into_py(py),
            Self::Float(f) => f.into_py(py),
            Self::String(s) => s.into_py(py),
            Self::Boolean(s) => s.into_py(py),
            Self::None(_) => <Option<Py<PyAny>>>::into_py(None, py),
        }
    }
}


fn plural_s(value: &usize) -> &'static str {
    if *value == 1 {
        ""
    } else {
        "s"
    }
}


#[enum_dispatch]
trait RenderPyMessage {
    fn render_python_message(&self, py: Python) -> PyResult<Py<PyString>>;
}

#[enum_dispatch]
trait RenderJsonMessage {
    fn render_json_message(&self, py: Python) -> PyResult<Py<PyString>>;
}

#[enum_dispatch]
trait GetErrorContext {
    fn get_context(&self, py: Python) -> PyResult<Py<PyAny>>;
}

#[enum_dispatch]
trait GetInput {
    fn get_input(&self) -> &Option<Py<PyAny>>;
}

#[enum_dispatch]
trait PyRichCompare {
    fn richcmp(&self, py: Python, other: &Self, op: CompareOp) -> PyResult<bool>;
}


#[derive(Debug, Clone)]
#[pyclass(extends = PyException, subclass, unsendable, module = "pydantic_core._pydantic_core")]
pub struct PydanticErrorBase;


macro_rules! make_error {
    ($name:ident { $($fname:ident : $ftype:ty),* $(,)?}, $message_template:expr $(, $init_stmt:stmt)?) => {
        #[derive(Debug, Clone, Serialize)]
        #[pyclass(extends = PydanticErrorBase, unsendable, module = "pydantic_core._pydantic_core")]
        pub struct $name {
            $(
                #[pyo3(get)]
                $fname: $ftype,
            )*
            #[serde(skip_serializing)]
            input: Option<Py<PyAny>>,
        }

        impl $name {
            pub fn new(
                $($fname: $ftype,)*
                input: Option<Py<PyAny>>,
            ) -> Self {
                Self {
                    $($fname,)*
                    input,
                }
            }
            #[allow(unused_variables)]
            fn eq(&self, other: &Self) -> bool {
                true
                $(&& self.$fname == other.$fname)*
            }
        }

        impl PyRichCompare for $name {
            fn richcmp(&self, _py: Python, other: &Self, op: CompareOp) -> PyResult<bool> {
                match op {
                    CompareOp::Eq => Ok(self.eq(other)),
                    CompareOp::Ne => Ok(!self.eq(other)),
                    _ => Err(PyNotImplementedError::new_err("")),
                }
            }
        }

        #[pymethods]
        impl $name {
            #[new]
            fn py_new(
                $($fname: $ftype,)*
            ) -> PyResult<PyClassInitializer<Self>> {
                Ok(PyClassInitializer::from(PydanticErrorBase).add_subclass(
                    Self {
                        $($fname,)*
                        input: None,
                    }
                ))
            }

            #[staticmethod]
            fn with_input(
                py: Python,
                $($fname: $ftype,)*
                input: Py<PyAny>
            ) -> PyResult<Py<Self>> {
                Py::new(
                    py,
                    PyClassInitializer::from(PydanticErrorBase).add_subclass(
                        Self {
                            $($fname,)*
                            input: Some(input),
                        }
                    )
                )
            }
            
            fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> PyResult<bool> {
                self.richcmp(py, other, op)
            }

            fn __str__(&self, py: Python) -> PyResult<String> {
                self.__repr__(py)
            }
            
            #[allow(unused_mut)]
            fn __repr__(&self, _py: Python) -> PyResult<String> {
                let mut fields: Vec<String> = vec![];
                $(fields.push(format!("{}={:?}", stringify!($fname), &self.$fname));)*
                let args = fields.join(",");
                Ok(format!("ValidationException({args})"))
            }
        }

        impl GetErrorContext for $name {
            fn get_context(&self, py: Python) -> PyResult<Py<PyAny>> {
                Ok(pythonize(py, self).unwrap())
            }
        }

        impl GetInput for $name {
            fn get_input(&self) -> &Option<Py<PyAny>> {
                &self.input
            }
        }

        impl RenderPyMessage for $name {
            fn render_python_message(&self, py: Python) -> PyResult<Py<PyString>> {
                $(let $fname = &self.$fname.clone();)*
                $($init_stmt)?
                let message = format!($message_template);
                Ok(PyString::new(py, &message).into())
            }
        }
    };
}

macro_rules! render_json_message_from_py_message {
    ($name:ident ) => {
        impl RenderJsonMessage for $name {
            fn render_json_message(&self, py: Python) -> PyResult<Py<PyString>> {
                self.render_python_message(py)
            }
        }
    };
}

macro_rules! render_fixed_json_message {
    ($name:ident, $message_template:expr ) => {
        impl RenderJsonMessage for $name {
            fn render_json_message(&self, py: Python) -> PyResult<Py<PyString>> {
                Ok(intern!(py, $message_template).into())
            }
        }
    };
}

make_error!(NoSuchAttributeError { attribute: String, }, "Object has no attribute '{attribute}'");
render_json_message_from_py_message!(NoSuchAttributeError);

make_error!(JsonInvalidError { error: String, }, "Invalid JSON: {error}");
render_json_message_from_py_message!(JsonInvalidError);

make_error!(JsonTypeError {}, "JSON input should be string, bytes or bytearray");
render_json_message_from_py_message!(JsonTypeError);

make_error!(RecursionLoopError {}, "Recursion error - cyclic reference detected");
render_json_message_from_py_message!(RecursionLoopError);

make_error!(DictAttributesTypeError {}, "Input should be a valid dictionary or instance to extract fields from");
render_json_message_from_py_message!(DictAttributesTypeError);

make_error!(MissingError {}, "Field required");
render_json_message_from_py_message!(MissingError);

make_error!(FrozenFieldError {}, "Field is frozen");
render_json_message_from_py_message!(FrozenFieldError);

make_error!(FrozenInstanceError {}, "Instance is frozen");
render_json_message_from_py_message!(FrozenInstanceError);

make_error!(ExtraForbiddenError {}, "Extra inputs are not permitted");
render_json_message_from_py_message!(ExtraForbiddenError);

make_error!(InvalidKeyError {}, "Keys should be strings");
render_json_message_from_py_message!(InvalidKeyError);

make_error!(GetAttributeError { error: String, }, "Error extracting attribute: {error}");
render_json_message_from_py_message!(GetAttributeError);

make_error!(ModelClassTypeError { class_name: String, }, "Input should be an instance of {class_name}");
render_json_message_from_py_message!(ModelClassTypeError);

make_error!(NoneRequiredError {}, "Input should be None");
render_json_message_from_py_message!(NoneRequiredError);

make_error!(BoolError {}, "Input should be a valid boolean");
render_json_message_from_py_message!(BoolError);

make_error!(GreaterThanError { gt: ContextValue, }, "Input should be greater than {gt}");
render_json_message_from_py_message!(GreaterThanError);

make_error!(GreaterThanEqualError { ge: ContextValue, }, "Input should be greater than or equal to {ge}");
render_json_message_from_py_message!(GreaterThanEqualError);

make_error!(LessThanError { lt: ContextValue, }, "Input should be less than {lt}");
render_json_message_from_py_message!(LessThanError);

make_error!(LessThanEqualError { le: ContextValue, }, "Input should be less than or equal to {le}");
render_json_message_from_py_message!(LessThanEqualError);

make_error!(MultipleOfError { multiple_of: ContextValue, }, "Input should be a multiple of {multiple_of}");
render_json_message_from_py_message!(MultipleOfError);

make_error!(FiniteNumberError {}, "Input should be a finite number");
render_json_message_from_py_message!(FiniteNumberError);

make_error!(TooShortError { field_type: String, min_length: usize, actual_length: usize, }, "{field_type} should have at least {min_length} item{expected_plural} after validation, not {actual_length}", let expected_plural = plural_s(min_length));
render_json_message_from_py_message!(TooShortError);

make_error!(TooLongError { field_type: String, max_length: usize, actual_length: usize, }, "{field_type} should have at most {max_length} item{expected_plural} after validation, not {actual_length}", let expected_plural = plural_s(max_length));
render_json_message_from_py_message!(TooLongError);

make_error!(IterableTypeError {}, "Input should be iterable");
render_json_message_from_py_message!(IterableTypeError);

make_error!(IterationError { error: String, }, "Error iterating over object, error: {error}");
render_json_message_from_py_message!(IterationError);

make_error!(StringTypeError {}, "Input should be a valid string");
render_json_message_from_py_message!(StringTypeError);

make_error!(StringSubTypeError {}, "Input should be a string, not an instance of a subclass of str");
render_json_message_from_py_message!(StringSubTypeError);

make_error!(StringUnicodeError {}, "Input should be a valid string, unable to parse raw data as a unicode string");
render_json_message_from_py_message!(StringUnicodeError);

make_error!(StringTooShortError { min_length: usize }, "String should have at least {min_length} characters");
render_json_message_from_py_message!(StringTooShortError);

make_error!(StringTooLongError { max_length: usize }, "String should have at most {max_length} characters");
render_json_message_from_py_message!(StringTooLongError);

make_error!(StringPatternMismatchError { pattern: String }, "String should match pattern '{pattern}'");
render_json_message_from_py_message!(StringPatternMismatchError);

make_error!(DictTypeError {}, "Input should be a valid dictionary");
render_json_message_from_py_message!(DictTypeError);

make_error!(MappingTypeError { error: String }, "Input should be a valid mapping, error: {error}");
render_json_message_from_py_message!(MappingTypeError);

make_error!(ListTypeError {}, "Input should be a valid list");
render_fixed_json_message!(ListTypeError, "Input should be a valid array");

make_error!(TupleTypeError {}, "Input should be a valid tuple");
render_json_message_from_py_message!(TupleTypeError);

make_error!(SetTypeError {}, "Input should be a valid set");
render_json_message_from_py_message!(SetTypeError);

make_error!(BoolTypeError {}, "Input should be a valid boolean");
render_json_message_from_py_message!(BoolTypeError);

make_error!(BoolParsingError {}, "Input should be a valid boolean, unable to interpret input");
render_json_message_from_py_message!(BoolParsingError);

make_error!(IntTypeError {}, "Input should be a valid integer");
render_json_message_from_py_message!(IntTypeError);

make_error!(IntParsingError {}, "Input should be a valid integer, unable to parse string as an integer");
render_json_message_from_py_message!(IntParsingError);

make_error!(IntFromFloatError {}, "Input should be a valid integer, got a number with a fractional part");
render_json_message_from_py_message!(IntFromFloatError);

make_error!(FloatTypeError {}, "Input should be a valid number");
render_json_message_from_py_message!(FloatTypeError);

make_error!(FloatParsingError {}, "Input should be a valid number, unable to parse string as an number");
render_json_message_from_py_message!(FloatParsingError);

make_error!(BytesTypeError {}, "Input should be a valid bytes");
render_json_message_from_py_message!(BytesTypeError);

make_error!(BytesTooShortError { min_length: usize }, "Data should have at least {min_length} bytes");
render_json_message_from_py_message!(BytesTooShortError);

make_error!(BytesTooLongError { max_length: usize }, "Data should have at most {max_length} bytes");
render_json_message_from_py_message!(BytesTooLongError);

make_error!(ValueError { error: String }, "Value error, {error}");
render_json_message_from_py_message!(ValueError);

make_error!(AssertionError { error: String }, "Assertion failed, {error}");
render_json_message_from_py_message!(AssertionError);

make_error!(LiteralError { expected: String }, "Input should be {expected}");
render_json_message_from_py_message!(LiteralError);

make_error!(DateTypeError {}, "Input should be a valid date");
render_json_message_from_py_message!(DateTypeError);

make_error!(DateParsingError { error: String }, "Input should be a valid date in the format YYYY-MM-DD, {error}");
render_json_message_from_py_message!(DateParsingError);

make_error!(DateFromDatetimeParsingError { error: String }, "Input should be a valid date or datetime, {error}");
render_json_message_from_py_message!(DateFromDatetimeParsingError);

make_error!(DateFromDatetimeInexactError {}, "Datetimes provided to dates should have zero time - e.g. be exact dates");
render_json_message_from_py_message!(DateFromDatetimeInexactError);

make_error!(DatePastError {}, "Date should be in the past");
render_json_message_from_py_message!(DatePastError);

make_error!(DateFutureError {}, "Date should be in the future");
render_json_message_from_py_message!(DateFutureError);

make_error!(TimeTypeError {}, "Input should be a valid time");
render_json_message_from_py_message!(TimeTypeError);

make_error!(TimeParsingError { error: String }, "Input should be in a valid time format, {error}");
render_json_message_from_py_message!(TimeParsingError);

make_error!(DatetimeTypeError {}, "Input should be a valid datetime");
render_json_message_from_py_message!(DatetimeTypeError);

make_error!(DatetimeParsingError { error: String }, "Input should be a valid datetime, {error}");
render_json_message_from_py_message!(DatetimeParsingError);

make_error!(DatetimeObjectInvalidError { error: String }, "Invalid datetime object, got {error}");
render_json_message_from_py_message!(DatetimeObjectInvalidError);

make_error!(DatetimePastError {}, "Datetime should be in the past");
render_json_message_from_py_message!(DatetimePastError);

make_error!(DatetimeFutureError {}, "Datetime should be in the future");
render_json_message_from_py_message!(DatetimeFutureError);

make_error!(DatetimeAwareError {}, "Datetime should have timezone info");
render_json_message_from_py_message!(DatetimeAwareError);

make_error!(DatetimeNaiveError {}, "Datetime should not have timezone info");
render_json_message_from_py_message!(DatetimeNaiveError);

make_error!(TimeDeltaTypeError {}, "Input should be a valid timedelta");
render_json_message_from_py_message!(TimeDeltaTypeError);

make_error!(TimeDeltaParsingError { error: String }, "Input should be a valid timedelta, {error}");
render_json_message_from_py_message!(TimeDeltaParsingError);

make_error!(FrozenSetTypeError {}, "Input should be a valid frozenset");
render_json_message_from_py_message!(FrozenSetTypeError);

make_error!(IsInstanceOfError { class: String }, "Input should be an instance of {class}");
render_json_message_from_py_message!(IsInstanceOfError);

make_error!(IsSubclassOfError { class: String }, "Input should be a subclass of {class}");
render_json_message_from_py_message!(IsSubclassOfError);

make_error!(CallableTypeError {}, "Input should be callable");
render_json_message_from_py_message!(CallableTypeError);

make_error!(UnionTagInvalidError { discriminator: String, tag: String, expected_tags: String }, "Input tag '{tag}' found using {discriminator} does not match any of the expected tags: {expected_tags}");
render_json_message_from_py_message!(UnionTagInvalidError);

make_error!(UnionTagNotFoundError { discriminator: String }, "Unable to extract tag using discriminator {discriminator}");
render_json_message_from_py_message!(UnionTagNotFoundError);

make_error!(ArgumentsTypeError {}, "Arguments must be a tuple, list or a dictionary");
render_json_message_from_py_message!(ArgumentsTypeError);

make_error!(MissingArgumentError {}, "Missing required argument");
render_json_message_from_py_message!(MissingArgumentError);

make_error!(UnexpectedKeywordArgumentError {}, "Unexpected keyword argument");
render_json_message_from_py_message!(UnexpectedKeywordArgumentError);

make_error!(MissingKeywordOnlyArgumentError {}, "Missing required keyword only argument");
render_json_message_from_py_message!(MissingKeywordOnlyArgumentError);

make_error!(UnexpectedPositionalArgumentError {}, "Unexpected positional argument");
render_json_message_from_py_message!(UnexpectedPositionalArgumentError);

make_error!(MissingPositionalOnlyArgumentError {}, "Missing required positional only argument");
render_json_message_from_py_message!(MissingPositionalOnlyArgumentError);

make_error!(MultipleArgumentValuesError {}, "Got multiple values for argument");
render_json_message_from_py_message!(MultipleArgumentValuesError);

make_error!(DataclassTypeError { dataclass_name: String }, "Input should be a dictionary or an instance of {dataclass_name}");
render_fixed_json_message!(DataclassTypeError, "Input should be an object");

make_error!(UrlTypeError {}, "URL input should be a string or URL");
render_json_message_from_py_message!(UrlTypeError);

make_error!(UrlParsingError { error: String }, "Input should be a valid URL, {error}");
render_json_message_from_py_message!(UrlParsingError);

make_error!(UrlSyntaxViolationError { error: String }, "Input violated strict URL syntax rules, {error}");
render_json_message_from_py_message!(UrlSyntaxViolationError);

make_error!(UrlTooLongError { max_length: usize }, "URL should have at most {max_length} characters");
render_json_message_from_py_message!(UrlTooLongError);

make_error!(UrlSchemeError { expected_schemes: String }, "URL scheme should be {expected_schemes}");
render_json_message_from_py_message!(UrlSchemeError);



#[derive(Debug, Clone)]
#[pyclass(extends = PydanticErrorBase, unsendable, module = "pydantic_core._pydantic_core")]
pub struct CustomError {
    message_template: String,
    error_type: String,
    context: HashMap<String, ContextValue>,
    input: Option<Py<PyAny>>,
    json_message_template: Option<String>,
}

impl CustomError {
    pub fn get_error_type(&self) -> String {
        self.error_type.clone()
    }
    fn format_py_string_with_context(&self, py: Python, s: &PyString) -> PyResult<Py<PyString>> {
        let ctx: Py<PyDict> = self.get_context(py)?.extract(py)?;
        match s.call_method( intern!(py, "format"), (), Some(ctx.as_ref(py))) {
            Ok(v) => Ok(v.extract::<&PyString>()?.into()),
            Err(e) => Err(e),
        }
    }
}


impl GetErrorContext for CustomError {
    fn get_context(&self, py:Python) -> PyResult<Py<PyAny>> {
        let ctx = PyDict::new(py);
        for (k, v) in self.context.clone() {
            ctx.set_item(k, v.into_py(py))?;
        }
        Ok(ctx.into())
    }
}


#[pymethods]
impl CustomError {
    #[new]
    fn py_new(
        message_template: String,
        error_type: String,
        context: HashMap<String, ContextValue>,
        json_message_template: Option<String>,
    ) -> PyResult<PyClassInitializer<Self>> {
        Ok(PyClassInitializer::from(PydanticErrorBase).add_subclass(
            Self {
                message_template,
                error_type,
                context,
                input: None,
                json_message_template,
            }
        ))
    }

    #[staticmethod]
    fn with_input(
        py: Python,
        message_template: String,
        error_type: String,
        context: HashMap<String, ContextValue>,
        input: Py<PyAny>,
        json_message_template: Option<String>,
    ) -> PyResult<Py<Self>> {
        Py::new(
            py,
            PyClassInitializer::from(PydanticErrorBase).add_subclass(
                Self {
                    message_template,
                    error_type,
                    context,
                    input: Some(input),
                    json_message_template,
                }
            )
        )
    }

    fn __richcmp__(&self, py: Python, other: &Self, op: CompareOp) -> PyResult<bool> {
        self.richcmp(py, other, op)
    }
}

impl PyRichCompare for CustomError {
    fn richcmp(&self, py: Python, other: &Self, op: CompareOp) -> PyResult<bool> {
        let fields_eq = self.message_template == other.message_template
            &&
            self.json_message_template == other.json_message_template
            &&
            self.error_type == other.error_type;
        let input_eq = match (&self.input, &other.input) {
            (None, None) => Ok(true),
            (None, Some(_)) => Ok(false),
            (Some(_), None) => Ok(false),
            (Some(l), Some(r)) => l.as_ref(py).eq(r),
        }?;
        let is_eq = fields_eq && input_eq;
        match op {
            CompareOp::Eq => Ok(is_eq),
            CompareOp::Ne => Ok(!is_eq),
            _ => Err(PyNotImplementedError::new_err(""))
        }
    }
}

impl GetInput for CustomError {
    fn get_input(&self) -> &Option<Py<PyAny>> {
        &self.input
    }
}

impl RenderPyMessage for CustomError {
    fn render_python_message(&self, py: Python) -> PyResult<Py<PyString>> {
        // Do python string formatting so that the result is predictable for users
        self.format_py_string_with_context(py, PyString::new(py, &self.message_template))
    }
}

impl RenderJsonMessage for CustomError {
    fn render_json_message(&self, py: Python) -> PyResult<Py<PyString>> {
        if let Some(s) = &self.json_message_template {
            self.format_py_string_with_context(py, PyString::new(py, s))
        } else {
            self.render_python_message(py)
        }
    }
}


#[enum_dispatch(RenderPyMessage)]
#[enum_dispatch(RenderJsonMessage)]
#[enum_dispatch(GetErrorContext)]
#[enum_dispatch(GetInput)]
#[derive(Clone, Debug, Display, FromPyObject)]
#[strum(serialize_all = "snake_case")]
pub enum PydanticException {
    NoSuchAttribute(NoSuchAttributeError),
    JsonInvalid(JsonInvalidError),
    JsonType(JsonTypeError),
    RecursionLoop(RecursionLoopError),
    DictAttributesType(DictAttributesTypeError),
    Missing(MissingError),
    FrozenField(FrozenFieldError),
    FrozenInstance(FrozenInstanceError),
    ExtraForbidden(ExtraForbiddenError),
    InvalidKey(InvalidKeyError),
    GetAttributeError(GetAttributeError),
    ModelClassType(ModelClassTypeError),
    NoneRequired(NoneRequiredError),
    Bool(BoolError),
    GreaterThan(GreaterThanError),
    GreaterThanEqual(GreaterThanEqualError),
    LessThan(LessThanError),
    LessThanEqual(LessThanEqualError),
    MultipleOf(MultipleOfError),
    FiniteNumber(FiniteNumberError),
    TooShort(TooShortError),
    TooLong(TooLongError),
    IterableType(IterableTypeError),
    IterationError(IterationError),
    StringType(StringTypeError),
    StringSubType(StringSubTypeError),
    StringUnicode(StringUnicodeError),
    StringTooShort(StringTooShortError),
    StringTooLong(StringTooLongError),
    StringPatternMismatch(StringPatternMismatchError),
    DictType(DictTypeError),
    MappingType(MappingTypeError),
    ListType(ListTypeError),
    TupleType(TupleTypeError),
    SetType(SetTypeError),
    BoolType(BoolTypeError),
    BoolParsing(BoolParsingError),
    IntType(IntTypeError),
    IntParsing(IntParsingError),
    IntFromFloat(IntFromFloatError),
    FloatType(FloatTypeError),
    FloatParsing(FloatParsingError),
    BytesType(BytesTypeError),
    BytesTooShort(BytesTooShortError),
    BytesTooLong(BytesTooLongError),
    ValueError(ValueError),
    AssertionError(AssertionError),
    LiteralError(LiteralError),
    DateType(DateTypeError),
    DateParsing(DateParsingError),
    DateFromDatetimeParsing(DateFromDatetimeParsingError),
    DateFromDatetimeInexact(DateFromDatetimeInexactError),
    DatePast(DatePastError),
    DateFuture(DateFutureError),
    TimeType(TimeTypeError),
    TimeParsing(TimeParsingError),
    DatetimeType(DatetimeTypeError),
    DatetimeParsing(DatetimeParsingError),
    DatetimeObjectInvalid(DatetimeObjectInvalidError),
    DatetimePast(DatetimePastError),
    DatetimeFuture(DatetimeFutureError),
    DatetimeAware(DatetimeAwareError),
    DatetimeNaive(DatetimeNaiveError),
    TimeDeltaType(TimeDeltaTypeError),
    TimeDeltaParsing(TimeDeltaParsingError),
    FrozenSetType(FrozenSetTypeError),
    IsInstanceOf(IsInstanceOfError),
    IsSubclassOf(IsSubclassOfError),
    CallableType(CallableTypeError),
    UnionTagInvalid(UnionTagInvalidError),
    UnionTagNotFound(UnionTagNotFoundError),
    ArgumentsType(ArgumentsTypeError),
    MissingArgument(MissingArgumentError),
    UnexpectedKeywordArgument(UnexpectedKeywordArgumentError),
    MissingKeywordOnlyArgument(MissingKeywordOnlyArgumentError),
    UnexpectedPositionalArgument(UnexpectedPositionalArgumentError),
    MissingPositionalOnlyArgument(MissingPositionalOnlyArgumentError),
    MultipleArgumentValues(MultipleArgumentValuesError),
    DataclassType(DataclassTypeError),
    UrlType(UrlTypeError),
    UrlParsing(UrlParsingError),
    UrlSyntaxViolation(UrlSyntaxViolationError),
    UrlTooLong(UrlTooLongError),
    UrlScheme(UrlSchemeError),
    CustomError(CustomError),
}


impl PydanticException {
    pub fn as_py_dict(&self, py: Python) -> PyResult<Py<PyDict>> {
        let res = PyDict::new(py);
        res.set_item(intern!(py, "ctx"), self.get_context(py)?)?;
        if let Some(input) = self.get_input() {
            res.set_item(intern!(py, "input"), input)?
        }
        let error_type = self.get_error_type();
        res.set_item(intern!(py, "error_type"), error_type)?;
        Ok(res.into())
    }

    pub fn get_error_type(&self) -> String {
        match self {
            Self::CustomError(e) => e.get_error_type(),
            other => other.to_string()
        }
    }
}

pub fn register_errors_module(_py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    parent_module.add_class::<NoSuchAttributeError>()?;
    parent_module.add_class::<JsonInvalidError>()?;
    parent_module.add_class::<JsonTypeError>()?;
    parent_module.add_class::<RecursionLoopError>()?;
    parent_module.add_class::<DictAttributesTypeError>()?;
    parent_module.add_class::<MissingError>()?;
    parent_module.add_class::<FrozenFieldError>()?;
    parent_module.add_class::<FrozenInstanceError>()?;
    parent_module.add_class::<ExtraForbiddenError>()?;
    parent_module.add_class::<InvalidKeyError>()?;
    parent_module.add_class::<GetAttributeError>()?;
    parent_module.add_class::<ModelClassTypeError>()?;
    parent_module.add_class::<NoneRequiredError>()?;
    parent_module.add_class::<BoolError>()?;
    parent_module.add_class::<GreaterThanError>()?;
    parent_module.add_class::<GreaterThanEqualError>()?;
    parent_module.add_class::<LessThanError>()?;
    parent_module.add_class::<LessThanEqualError>()?;
    parent_module.add_class::<MultipleOfError>()?;
    parent_module.add_class::<FiniteNumberError>()?;
    parent_module.add_class::<TooShortError>()?;
    parent_module.add_class::<TooLongError>()?;
    parent_module.add_class::<IterableTypeError>()?;
    parent_module.add_class::<IterationError>()?;
    parent_module.add_class::<StringTypeError>()?;
    parent_module.add_class::<StringSubTypeError>()?;
    parent_module.add_class::<StringUnicodeError>()?;
    parent_module.add_class::<StringTooShortError>()?;
    parent_module.add_class::<StringTooLongError>()?;
    parent_module.add_class::<StringPatternMismatchError>()?;
    parent_module.add_class::<DictTypeError>()?;
    parent_module.add_class::<MappingTypeError>()?;
    parent_module.add_class::<ListTypeError>()?;
    parent_module.add_class::<TupleTypeError>()?;
    parent_module.add_class::<SetTypeError>()?;
    parent_module.add_class::<BoolTypeError>()?;
    parent_module.add_class::<BoolParsingError>()?;
    parent_module.add_class::<IntTypeError>()?;
    parent_module.add_class::<IntParsingError>()?;
    parent_module.add_class::<IntFromFloatError>()?;
    parent_module.add_class::<FloatTypeError>()?;
    parent_module.add_class::<FloatParsingError>()?;
    parent_module.add_class::<BytesTypeError>()?;
    parent_module.add_class::<BytesTooShortError>()?;
    parent_module.add_class::<BytesTooLongError>()?;
    parent_module.add_class::<ValueError>()?;
    parent_module.add_class::<AssertionError>()?;
    parent_module.add_class::<LiteralError>()?;
    parent_module.add_class::<DateTypeError>()?;
    parent_module.add_class::<DateParsingError>()?;
    parent_module.add_class::<DateFromDatetimeParsingError>()?;
    parent_module.add_class::<DateFromDatetimeInexactError>()?;
    parent_module.add_class::<DatePastError>()?;
    parent_module.add_class::<DateFutureError>()?;
    parent_module.add_class::<TimeTypeError>()?;
    parent_module.add_class::<TimeParsingError>()?;
    parent_module.add_class::<DatetimeTypeError>()?;
    parent_module.add_class::<DatetimeParsingError>()?;
    parent_module.add_class::<DatetimeObjectInvalidError>()?;
    parent_module.add_class::<DatetimePastError>()?;
    parent_module.add_class::<DatetimeFutureError>()?;
    parent_module.add_class::<DatetimeAwareError>()?;
    parent_module.add_class::<DatetimeNaiveError>()?;
    parent_module.add_class::<TimeDeltaTypeError>()?;
    parent_module.add_class::<TimeDeltaParsingError>()?;
    parent_module.add_class::<FrozenSetTypeError>()?;
    parent_module.add_class::<IsInstanceOfError>()?;
    parent_module.add_class::<IsSubclassOfError>()?;
    parent_module.add_class::<CallableTypeError>()?;
    parent_module.add_class::<UnionTagInvalidError>()?;
    parent_module.add_class::<UnionTagNotFoundError>()?;
    parent_module.add_class::<ArgumentsTypeError>()?;
    parent_module.add_class::<MissingArgumentError>()?;
    parent_module.add_class::<UnexpectedKeywordArgumentError>()?;
    parent_module.add_class::<MissingKeywordOnlyArgumentError>()?;
    parent_module.add_class::<UnexpectedPositionalArgumentError>()?;
    parent_module.add_class::<MissingPositionalOnlyArgumentError>()?;
    parent_module.add_class::<MultipleArgumentValuesError>()?;
    parent_module.add_class::<DataclassTypeError>()?;
    parent_module.add_class::<UrlTypeError>()?;
    parent_module.add_class::<UrlParsingError>()?;
    parent_module.add_class::<UrlSyntaxViolationError>()?;
    parent_module.add_class::<UrlTooLongError>()?;
    parent_module.add_class::<UrlSchemeError>()?;
    Ok(())
}
