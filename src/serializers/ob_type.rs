use pyo3::ffi::PyTypeObject;
use pyo3::once_cell::GILOnceCell;
use pyo3::prelude::*;
use pyo3::types::{
    PyBool, PyByteArray, PyBytes, PyDate, PyDateTime, PyDelta, PyDict, PyFloat, PyFrozenSet, PyInt, PyIterator, PyList,
    PySet, PyString, PyTime, PyTuple,
};
use pyo3::{intern, AsPyPointer, PyTypeInfo};

use strum::Display;
use strum_macros::EnumString;

use crate::url::{PyMultiHostUrl, PyUrl};

#[derive(Debug, Clone)]
pub struct ObTypeLookup {
    // valid JSON types
    none: usize,
    int: usize,
    bool: usize,
    float: usize,
    string: usize,
    list: usize,
    dict: usize,
    // other numeric types
    decimal_object: PyObject,
    decimal: usize,
    // other string types
    bytes: usize,
    bytearray: usize,
    // other sequence types
    tuple: usize,
    set: usize,
    frozenset: usize,
    // datetime types
    datetime: usize,
    date: usize,
    time: usize,
    timedelta: usize,
    // types from this package
    url: usize,
    multi_host_url: usize,
    // enum type
    enum_object: PyObject,
    enum_: usize,
    // generator
    generator_object: PyObject,
    generator: usize,
    // path
    path_object: PyObject,
    path: usize,
    // uuid type
    uuid_object: PyObject,
    uuid: usize,
}

static TYPE_LOOKUP: GILOnceCell<ObTypeLookup> = GILOnceCell::new();

#[derive(Debug)]
pub enum IsType {
    Exact,
    Subclass,
    False,
}

impl ObTypeLookup {
    fn new(py: Python) -> Self {
        let lib_url = url::Url::parse("https://example.com").unwrap();
        let decimal_type = py.import("decimal").unwrap().getattr("Decimal").unwrap();
        let enum_object = py.import("enum").unwrap().getattr("Enum").unwrap();
        let generator_object = py.import("types").unwrap().getattr("GeneratorType").unwrap();
        let path_object = py.import("pathlib").unwrap().getattr("Path").unwrap();
        let uuid_object = py.import("uuid").unwrap().getattr("UUID").unwrap();
        Self {
            none: py.None().as_ref(py).get_type_ptr() as usize,
            int: 0i32.into_py(py).as_ref(py).get_type_ptr() as usize,
            bool: true.into_py(py).as_ref(py).get_type_ptr() as usize,
            float: 0f32.into_py(py).as_ref(py).get_type_ptr() as usize,
            list: PyList::empty(py).get_type_ptr() as usize,
            dict: PyDict::new(py).get_type_ptr() as usize,
            decimal_object: decimal_type.to_object(py),
            decimal: decimal_type.as_ptr() as usize,
            string: PyString::new(py, "s").get_type_ptr() as usize,
            bytes: PyBytes::new(py, b"s").get_type_ptr() as usize,
            bytearray: PyByteArray::new(py, b"s").get_type_ptr() as usize,
            tuple: PyTuple::empty(py).get_type_ptr() as usize,
            set: PySet::empty(py).unwrap().get_type_ptr() as usize,
            frozenset: PyFrozenSet::empty(py).unwrap().get_type_ptr() as usize,
            datetime: PyDateTime::new(py, 2000, 1, 1, 0, 0, 0, 0, None)
                .unwrap()
                .get_type_ptr() as usize,
            date: PyDate::new(py, 2000, 1, 1).unwrap().get_type_ptr() as usize,
            time: PyTime::new(py, 0, 0, 0, 0, None).unwrap().get_type_ptr() as usize,
            timedelta: PyDelta::new(py, 0, 0, 0, false).unwrap().get_type_ptr() as usize,
            url: PyUrl::new(lib_url.clone()).into_py(py).as_ref(py).get_type_ptr() as usize,
            multi_host_url: PyMultiHostUrl::new(lib_url, None).into_py(py).as_ref(py).get_type_ptr() as usize,
            enum_object: enum_object.to_object(py),
            enum_: enum_object.get_type_ptr() as usize,
            generator_object: generator_object.to_object(py),
            generator: generator_object.as_ptr() as usize,
            path_object: path_object.to_object(py),
            path: path_object.as_ptr() as usize,
            uuid_object: uuid_object.to_object(py),
            uuid: uuid_object.as_ptr() as usize,
        }
    }

    pub fn cached(py: Python<'_>) -> &Self {
        TYPE_LOOKUP.get_or_init(py, || Self::new(py))
    }

    pub fn is_type(&self, value: &PyAny, expected_ob_type: ObType) -> IsType {
        match self.ob_type_is_expected(Some(value), value.get_type_ptr(), &expected_ob_type) {
            IsType::False => {
                if expected_ob_type == self.fallback_isinstance(value) {
                    IsType::Subclass
                } else {
                    IsType::False
                }
            }
            is_type => is_type,
        }
    }

    fn ob_type_is_expected(
        &self,
        op_value: Option<&PyAny>,
        type_ptr: *mut PyTypeObject,
        expected_ob_type: &ObType,
    ) -> IsType {
        let ob_type = type_ptr as usize;
        let ans = match expected_ob_type {
            ObType::None => self.none == ob_type,
            ObType::Int => self.int == ob_type,
            // op_value is None on recursive calls
            ObType::IntSubclass => self.int == ob_type && op_value.is_none(),
            ObType::Bool => self.bool == ob_type,
            ObType::Float => {
                if self.float == ob_type {
                    true
                } else if self.int == ob_type {
                    // special case for int as the input to float serializer,
                    // https://github.com/pydantic/pydantic-core/pull/866
                    return IsType::Subclass;
                } else {
                    false
                }
            }
            ObType::FloatSubclass => self.float == ob_type && op_value.is_none(),
            ObType::Str => self.string == ob_type,
            ObType::List => self.list == ob_type,
            ObType::Dict => self.dict == ob_type,
            ObType::Decimal => self.decimal == ob_type,
            ObType::StrSubclass => self.string == ob_type && op_value.is_none(),
            ObType::Tuple => self.tuple == ob_type,
            ObType::Set => self.set == ob_type,
            ObType::Frozenset => self.frozenset == ob_type,
            ObType::Bytes => self.bytes == ob_type,
            ObType::Datetime => self.datetime == ob_type,
            ObType::Date => self.date == ob_type,
            ObType::Time => self.time == ob_type,
            ObType::Timedelta => self.timedelta == ob_type,
            ObType::Bytearray => self.bytearray == ob_type,
            ObType::Url => self.url == ob_type,
            ObType::MultiHostUrl => self.multi_host_url == ob_type,
            ObType::Dataclass => is_dataclass(op_value),
            ObType::PydanticSerializable => is_pydantic_serializable(op_value),
            ObType::Enum => self.enum_ == ob_type,
            ObType::Generator => self.generator == ob_type,
            ObType::Path => self.path == ob_type,
            ObType::Uuid => self.uuid == ob_type,
            ObType::Unknown => false,
        };

        if ans {
            IsType::Exact
        } else {
            // this allows for subtypes of the supported class types,
            // if we didn't successfully confirm the type, we try again with the next base type pointer provided
            // it's not null
            let base_type_ptr = unsafe { (*type_ptr).tp_base };
            if base_type_ptr.is_null() {
                IsType::False
            } else {
                // as bellow, we don't want to tests for dataclass etc. again, so we pass None as op_value
                match self.ob_type_is_expected(None, base_type_ptr, expected_ob_type) {
                    IsType::False => IsType::False,
                    _ => IsType::Subclass,
                }
            }
        }
    }

    pub fn get_type(&self, value: &PyAny) -> ObType {
        match self.lookup_by_ob_type(Some(value), value.get_type_ptr()) {
            ObType::Unknown => self.fallback_isinstance(value),
            ob_type => ob_type,
        }
    }

    fn lookup_by_ob_type(&self, op_value: Option<&PyAny>, type_ptr: *mut PyTypeObject) -> ObType {
        let ob_type = type_ptr as usize;
        // this should be pretty fast, but still order is a bit important, so the most common types should come first
        // thus we don't follow the order of ObType
        if ob_type == self.none {
            ObType::None
        } else if ob_type == self.int {
            // op_value is None on recursive calls, e.g. hence the original value would be a subclass
            match op_value {
                Some(_) => ObType::Int,
                None => ObType::IntSubclass,
            }
        } else if ob_type == self.bool {
            ObType::Bool
        } else if ob_type == self.float {
            match op_value {
                Some(_) => ObType::Float,
                None => ObType::FloatSubclass,
            }
        } else if ob_type == self.string {
            match op_value {
                Some(_) => ObType::Str,
                None => ObType::StrSubclass,
            }
        } else if ob_type == self.list {
            ObType::List
        } else if ob_type == self.dict {
            ObType::Dict
        } else if ob_type == self.decimal {
            ObType::Decimal
        } else if ob_type == self.bytes {
            ObType::Bytes
        } else if ob_type == self.tuple {
            ObType::Tuple
        } else if ob_type == self.set {
            ObType::Set
        } else if ob_type == self.frozenset {
            ObType::Frozenset
        } else if ob_type == self.datetime {
            ObType::Datetime
        } else if ob_type == self.date {
            ObType::Date
        } else if ob_type == self.time {
            ObType::Time
        } else if ob_type == self.timedelta {
            ObType::Timedelta
        } else if ob_type == self.bytearray {
            ObType::Bytearray
        } else if ob_type == self.url {
            ObType::Url
        } else if ob_type == self.multi_host_url {
            ObType::MultiHostUrl
        } else if ob_type == self.uuid {
            ObType::Uuid
        } else if is_pydantic_serializable(op_value) {
            ObType::PydanticSerializable
        } else if is_dataclass(op_value) {
            ObType::Dataclass
        } else if self.is_enum(op_value, type_ptr) {
            ObType::Enum
        } else if ob_type == self.generator || is_generator(op_value) {
            ObType::Generator
        } else if ob_type == self.path {
            ObType::Path
        } else {
            // this allows for subtypes of the supported class types,
            // if `ob_type` didn't match any member of self, we try again with the next base type pointer
            let base_type_ptr = unsafe { (*type_ptr).tp_base };
            if base_type_ptr.is_null() {
                ObType::Unknown
            } else {
                // we don't want to tests for dataclass etc. again, so we pass None as op_value
                self.lookup_by_ob_type(None, base_type_ptr)
            }
        }
    }

    fn is_enum(&self, op_value: Option<&PyAny>, type_ptr: *mut PyTypeObject) -> bool {
        // only test on the type itself, not base types
        if op_value.is_some() {
            // see https://github.com/PyO3/pyo3/issues/2905 for details
            #[cfg(all(PyPy, not(Py_3_9)))]
            let meta_type = unsafe { (*type_ptr).ob_type };
            #[cfg(any(not(PyPy), Py_3_9))]
            let meta_type = unsafe { (*type_ptr).ob_base.ob_base.ob_type };
            meta_type as usize == self.enum_
        } else {
            false
        }
    }

    /// If our logic for finding types by recursively checking `tp_base` fails, we fallback to this which
    /// uses `isinstance` thus supporting both mixins and classes that implement `__instancecheck__`.
    /// We care about order here since:
    /// 1. we pay a price for each `isinstance` call
    /// 2. some types are subclasses of others, e.g. `bool` is a subclass of `int`
    /// hence we put common types first
    fn fallback_isinstance(&self, value: &PyAny) -> ObType {
        let py = value.py();
        if PyBool::is_type_of(value) {
            ObType::Bool
        } else if PyInt::is_type_of(value) {
            ObType::IntSubclass
        } else if PyFloat::is_type_of(value) {
            ObType::FloatSubclass
        } else if PyString::is_type_of(value) {
            ObType::StrSubclass
        } else if PyBytes::is_type_of(value) {
            ObType::Bytes
        } else if PyByteArray::is_type_of(value) {
            ObType::Bytearray
        } else if PyList::is_type_of(value) {
            ObType::List
        } else if PyTuple::is_type_of(value) {
            ObType::Tuple
        } else if PyDict::is_type_of(value) {
            ObType::Dict
        } else if PySet::is_type_of(value) {
            ObType::Set
        } else if PyFrozenSet::is_type_of(value) {
            ObType::Frozenset
        } else if PyDateTime::is_type_of(value) {
            ObType::Datetime
        } else if PyDate::is_type_of(value) {
            ObType::Date
        } else if PyTime::is_type_of(value) {
            ObType::Time
        } else if PyDelta::is_type_of(value) {
            ObType::Timedelta
        } else if PyUrl::is_type_of(value) {
            ObType::Url
        } else if PyMultiHostUrl::is_type_of(value) {
            ObType::MultiHostUrl
        } else if value.is_instance(self.decimal_object.as_ref(py)).unwrap_or(false) {
            ObType::Decimal
        } else if value.is_instance(self.uuid_object.as_ref(py)).unwrap_or(false) {
            ObType::Uuid
        } else if value.is_instance(self.enum_object.as_ref(py)).unwrap_or(false) {
            ObType::Enum
        } else if value.is_instance(self.generator_object.as_ref(py)).unwrap_or(false) {
            ObType::Generator
        } else if value.is_instance(self.path_object.as_ref(py)).unwrap_or(false) {
            ObType::Path
        } else {
            ObType::Unknown
        }
    }
}

fn is_dataclass(op_value: Option<&PyAny>) -> bool {
    if let Some(value) = op_value {
        value
            .hasattr(intern!(value.py(), "__dataclass_fields__"))
            .unwrap_or(false)
    } else {
        false
    }
}

fn is_pydantic_serializable(op_value: Option<&PyAny>) -> bool {
    if let Some(value) = op_value {
        value
            .hasattr(intern!(value.py(), "__pydantic_serializer__"))
            .unwrap_or(false)
    } else {
        false
    }
}

fn is_generator(op_value: Option<&PyAny>) -> bool {
    if let Some(value) = op_value {
        value.downcast::<PyIterator>().is_ok()
    } else {
        false
    }
}

#[derive(Debug, Clone, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum ObType {
    None,
    // numeric types
    Int,
    IntSubclass,
    Bool,
    Float,
    FloatSubclass,
    Decimal,
    // string types
    Str,
    StrSubclass,
    Bytes,
    Bytearray,
    // sequence types
    List,
    Tuple,
    Set,
    Frozenset,
    // mapping types
    Dict,
    // datetime types
    Datetime,
    Date,
    Time,
    Timedelta,
    // types from this package
    Url,
    MultiHostUrl,
    // anything with __pydantic_serializer__, including BaseModel and pydantic dataclasses
    PydanticSerializable,
    // vanilla dataclasses
    Dataclass,
    // enum type
    Enum,
    // generator type
    Generator,
    // Path
    Path,
    // Uuid
    Uuid,
    // unknown type
    Unknown,
}

impl PartialEq for ObType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Int, Self::Int) => true,
            (Self::IntSubclass, Self::IntSubclass) => true,
            (Self::Bool, Self::Bool) => true,
            (Self::Float, Self::Float) => true,
            (Self::FloatSubclass, Self::FloatSubclass) => true,
            (Self::Decimal, Self::Decimal) => true,
            (Self::Str, Self::Str) => true,
            (Self::StrSubclass, Self::StrSubclass) => true,
            (Self::Bytes, Self::Bytes) => true,
            (Self::Bytearray, Self::Bytearray) => true,
            (Self::List, Self::List) => true,
            (Self::Tuple, Self::Tuple) => true,
            (Self::Set, Self::Set) => true,
            (Self::Frozenset, Self::Frozenset) => true,
            (Self::Dict, Self::Dict) => true,
            (Self::Datetime, Self::Datetime) => true,
            (Self::Date, Self::Date) => true,
            (Self::Time, Self::Time) => true,
            (Self::Timedelta, Self::Timedelta) => true,
            (Self::Url, Self::Url) => true,
            (Self::MultiHostUrl, Self::MultiHostUrl) => true,
            (Self::PydanticSerializable, Self::PydanticSerializable) => true,
            (Self::Dataclass, Self::Dataclass) => true,
            (Self::Enum, Self::Enum) => true,
            (Self::Generator, Self::Generator) => true,
            (Self::Path, Self::Path) => true,
            (Self::Uuid, Self::Uuid) => true,
            // not Unknown here - unknowns are never equal
            // special cases for subclasses
            (Self::IntSubclass, Self::Int) => true,
            (Self::Int, Self::IntSubclass) => true,
            (Self::FloatSubclass, Self::Float) => true,
            (Self::Float, Self::FloatSubclass) => true,
            (Self::StrSubclass, Self::Str) => true,
            (Self::Str, Self::StrSubclass) => true,
            _ => false,
        }
    }
}
