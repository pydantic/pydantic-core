use std::borrow::Cow;
use std::sync::Arc;

use pyo3::exceptions::{PyAttributeError, PyRecursionError, PyRuntimeError};
use pyo3::gc::PyVisit;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::PyTraverseError;

use pyo3::types::PyString;

use crate::definitions::DefinitionsBuilder;
use crate::tools::SchemaDict;
use crate::tools::{function_name, py_err, py_error_type};
use crate::{PydanticOmit, PydanticSerializationUnexpectedValue};

use super::format::WhenUsed;

use super::any::AnySerializer;
use super::{
    infer_json_key, infer_serialize, infer_to_python, py_err_se_err, AnyFilter, BuildSerializer, CombinedSerializer,
    Extra, ExtraOwned, PydanticSerializationError, SerMode, TypeSerializer,
};

pub struct FunctionBeforeSerializerBuilder;

impl BuildSerializer for FunctionBeforeSerializerBuilder {
    const EXPECTED_TYPE: &'static str = "function-before";

    fn build(
        schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        // `before` schemas will obviously have type from `schema` since the validator is called second
        let schema = schema.get_as_req(intern!(py, "schema"))?;
        CombinedSerializer::build(&schema, config, definitions)
    }
}

pub struct FunctionAfterSerializerBuilder;

impl BuildSerializer for FunctionAfterSerializerBuilder {
    const EXPECTED_TYPE: &'static str = "function-after";
    fn build(
        schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        // While `before` function schemas do not modify the output type (and therefore affect the
        // serialization), for `after` schemas, there's no way to directly infer what schema should
        // be used for serialization. For convenience, the default is to assume the wrapped schema
        // should be used; the user/lib can override the serializer if necessary.
        let schema = schema.get_as_req(intern!(py, "schema"))?;
        CombinedSerializer::build(&schema, config, definitions)
    }
}

pub struct FunctionPlainSerializerBuilder;

impl BuildSerializer for FunctionPlainSerializerBuilder {
    const EXPECTED_TYPE: &'static str = "function-plain";
    fn build(
        schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        super::any::AnySerializer::build(schema, config, definitions)
    }
}

#[derive(Debug)]
pub struct FunctionPlainSerializer {
    func: PyObject,
    name: String,
    function_name: String,
    return_serializer: Box<CombinedSerializer>,
    // fallback serializer - used when when_used decides that this serializer should not be used
    fallback_serializer: Option<Box<CombinedSerializer>>,
    when_used: WhenUsed,
    is_field_serializer: bool,
    info_arg: bool,
}

fn destructure_function_schema<'py>(schema: &Bound<'py, PyDict>) -> PyResult<(bool, bool, Bound<'py, PyAny>)> {
    let function = schema.get_as_req(intern!(schema.py(), "function"))?;
    let is_field_serializer: bool = schema
        .get_as(intern!(schema.py(), "is_field_serializer"))?
        .unwrap_or(false);
    let info_arg: bool = schema.get_as(intern!(schema.py(), "info_arg"))?.unwrap_or(false);
    Ok((is_field_serializer, info_arg, function))
}

impl BuildSerializer for FunctionPlainSerializer {
    const EXPECTED_TYPE: &'static str = "function-plain";

    /// NOTE! `schema` here is the actual `CoreSchema`, not `schema.serialization` as in the other builders
    /// (done this way to match `FunctionWrapSerializer` which requires the full schema)
    fn build(
        schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();

        let ser_schema = schema.get_as_req(intern!(py, "serialization"))?;

        let (is_field_serializer, info_arg, function) = destructure_function_schema(&ser_schema)?;
        let function_name = function_name(&function)?;

        let return_serializer = match ser_schema.get_as(intern!(py, "return_schema"))? {
            Some(s) => Box::new(CombinedSerializer::build(&s, config, definitions)?),
            None => Box::new(AnySerializer::build(schema, config, definitions)?),
        };

        let when_used = WhenUsed::new(&ser_schema, WhenUsed::Always)?;
        let fallback_serializer = match when_used {
            WhenUsed::Always => None,
            _ => {
                let new_schema = copy_outer_schema(schema)?;
                Some(Box::new(CombinedSerializer::build(&new_schema, config, definitions)?))
            }
        };

        let name = format!("plain_function[{function_name}]");
        Ok(Self {
            func: function.unbind(),
            function_name,
            name,
            return_serializer,
            fallback_serializer,
            when_used,
            is_field_serializer,
            info_arg,
        }
        .into())
    }
}

impl FunctionPlainSerializer {
    fn call(
        &self,
        value: &Bound<'_, PyAny>,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<(bool, PyObject)> {
        let py = value.py();
        if self.when_used.should_use(value, extra) {
            let v = if self.is_field_serializer {
                if let Some(model) = extra.model {
                    if self.info_arg {
                        let info = SerializationInfo::new(include, exclude, extra, self.is_field_serializer)?;
                        self.func.call1(py, (model, value, info))?
                    } else {
                        self.func.call1(py, (model, value))?
                    }
                } else {
                    return Err(PyRuntimeError::new_err("Function plain serializer expected to be run inside the context of a model field but no model was found"));
                }
            } else if self.info_arg {
                let info = SerializationInfo::new(include, exclude, extra, self.is_field_serializer)?;
                self.func.call1(py, (value, info))?
            } else {
                self.func.call1(py, (value,))?
            };
            Ok((true, v))
        } else {
            Ok((false, value.clone().unbind()))
        }
    }

    fn get_fallback_serializer(&self) -> &CombinedSerializer {
        self.fallback_serializer
            .as_ref()
            .expect("fallback_serializer unexpectedly none")
            .as_ref()
    }

    fn retry_with_lax_check(&self) -> bool {
        self.fallback_serializer
            .as_ref()
            .is_some_and(|f| f.retry_with_lax_check())
            || self.return_serializer.retry_with_lax_check()
    }
}

fn on_error(py: Python, err: PyErr, function_name: &str, extra: &Extra) -> PyResult<()> {
    let exception = err.value(py);
    if let Ok(ser_err) = exception.extract::<PydanticSerializationUnexpectedValue>() {
        if extra.check.enabled() {
            Err(err)
        } else {
            extra.warnings.register_warning(ser_err);
            Ok(())
        }
    } else if let Ok(err) = exception.extract::<PydanticSerializationError>() {
        py_err!(PydanticSerializationError; "{}", err)
    } else if exception.is_instance_of::<PyRecursionError>() {
        py_err!(PydanticSerializationError; "Error calling function `{}`: RecursionError", function_name)
    } else {
        let new_err = py_error_type!(PydanticSerializationError; "Error calling function `{}`: {}", function_name, err);
        new_err.set_cause(py, Some(err));
        Err(new_err)
    }
}

macro_rules! function_type_serializer {
    ($name:ident) => {
        impl TypeSerializer for $name {
            fn to_python(
                &self,
                value: &Bound<'_, PyAny>,
                include: Option<&Bound<'_, PyAny>>,
                exclude: Option<&Bound<'_, PyAny>>,
                extra: &Extra,
            ) -> PyResult<PyObject> {
                let py = value.py();
                match self.call(value, include, exclude, extra) {
                    // None for include/exclude here, as filtering should be done
                    Ok((true, v)) => self.return_serializer.to_python(v.bind(py), None, None, extra),
                    Ok((false, v)) => self
                        .get_fallback_serializer()
                        .to_python(v.bind(py), None, None, extra),
                    Err(err) => {
                        on_error(py, err, &self.function_name, extra)?;
                        infer_to_python(value, include, exclude, extra)
                    }
                }
            }

            fn json_key<'a>(&self, key: &'a Bound<'_, PyAny>, extra: &Extra) -> PyResult<Cow<'a, str>> {
                let py = key.py();
                match self.call(key, None, None, extra) {
                    Ok((true, v)) => self
                        .return_serializer
                        .json_key(v.bind(py), extra)
                        .map(|cow| Cow::Owned(cow.into_owned())),
                    Ok((false, v)) => self
                        .get_fallback_serializer()
                        .json_key(v.bind(py), extra)
                        .map(|cow| Cow::Owned(cow.into_owned())),
                    Err(err) => {
                        on_error(py, err, &self.function_name, extra)?;
                        infer_json_key(key, extra)
                    }
                }
            }

            fn serde_serialize<S: serde::ser::Serializer>(
                &self,
                value: &Bound<'_, PyAny>,
                serializer: S,
                include: Option<&Bound<'_, PyAny>>,
                exclude: Option<&Bound<'_, PyAny>>,
                extra: &Extra,
            ) -> Result<S::Ok, S::Error> {
                let py = value.py();
                match self.call(value, include, exclude, extra) {
                    // None for include/exclude here, as filtering should be done
                    Ok((true, v)) => self
                        .return_serializer
                        .serde_serialize(v.bind(py), serializer, None, None, extra),
                    Ok((false, v)) => {
                        self.get_fallback_serializer()
                            .serde_serialize(v.bind(py), serializer, None, None, extra)
                    }
                    Err(err) => {
                        on_error(py, err, &self.function_name, extra).map_err(py_err_se_err)?;
                        infer_serialize(value, serializer, include, exclude, extra)
                    }
                }
            }

            fn get_name(&self) -> &str {
                &self.name
            }

            fn retry_with_lax_check(&self) -> bool {
                self.retry_with_lax_check()
            }
        }
    };
}

impl_py_gc_traverse!(FunctionPlainSerializer {
    func,
    return_serializer,
    fallback_serializer
});

function_type_serializer!(FunctionPlainSerializer);

fn copy_outer_schema<'py>(schema: &Bound<'py, PyDict>) -> PyResult<Bound<'py, PyDict>> {
    let py = schema.py();
    // we copy the schema so we can modify it without affecting the original
    let schema_copy = schema.copy()?;
    // remove the serialization key from the schema so we don't recurse
    schema_copy.del_item(intern!(py, "serialization"))?;
    // remove ref if it exists - the point is that `schema` here has already run through
    // `CombinedSerializer::build` so "ref" here will have already been added to `Definitions::used_ref`
    // we don't want to error by "finding" it now
    schema_copy.del_item(intern!(py, "ref")).ok();
    Ok(schema_copy)
}

pub struct FunctionWrapSerializerBuilder;

impl BuildSerializer for FunctionWrapSerializerBuilder {
    const EXPECTED_TYPE: &'static str = "function-wrap";
    fn build(
        schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        // While `before` function schemas do not modify the output type (and therefore affect the
        // serialization), for `wrap` schemas (like `after`), there's no way to directly infer what
        // schema should be used for serialization. For convenience, the default is to assume the
        // wrapped schema should be used; the user/lib can override the serializer if necessary.
        let schema = schema.get_as_req(intern!(py, "schema"))?;
        CombinedSerializer::build(&schema, config, definitions)
    }
}

#[derive(Debug)]
pub struct FunctionWrapSerializer {
    serializer: Arc<CombinedSerializer>,
    func: PyObject,
    name: String,
    function_name: String,
    return_serializer: Arc<CombinedSerializer>,
    when_used: WhenUsed,
    is_field_serializer: bool,
    info_arg: bool,
}

impl BuildSerializer for FunctionWrapSerializer {
    const EXPECTED_TYPE: &'static str = "function-wrap";

    /// NOTE! `schema` here is the actual `CoreSchema`, not `schema.serialization` as in the other builders
    /// (done this way since we need the `CoreSchema`)
    fn build(
        schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedSerializer>,
    ) -> PyResult<CombinedSerializer> {
        let py = schema.py();
        let ser_schema = schema.get_as_req(intern!(py, "serialization"))?;

        let (is_field_serializer, info_arg, function) = destructure_function_schema(&ser_schema)?;
        let function_name = function_name(&function)?;

        // try to get `schema.serialization.schema`, otherwise use `schema` with `serialization` key removed
        let inner_schema = if let Some(s) = ser_schema.get_as(intern!(py, "schema"))? {
            s
        } else {
            copy_outer_schema(schema)?
        };

        let serializer = CombinedSerializer::build(&inner_schema, config, definitions)?;

        let return_serializer = match ser_schema.get_as(intern!(py, "return_schema"))? {
            Some(s) => CombinedSerializer::build(&s, config, definitions)?,
            None => AnySerializer::build(schema, config, definitions)?,
        };

        let name = format!("wrap_function[{function_name}, {}]", serializer.get_name());
        Ok(Self {
            serializer: Arc::new(serializer),
            func: function.into(),
            function_name,
            name,
            return_serializer: Arc::new(return_serializer),
            when_used: WhenUsed::new(&ser_schema, WhenUsed::Always)?,
            is_field_serializer,
            info_arg,
        }
        .into())
    }
}

impl FunctionWrapSerializer {
    fn call(
        &self,
        value: &Bound<'_, PyAny>,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> PyResult<(bool, PyObject)> {
        let py = value.py();
        if self.when_used.should_use(value, extra) {
            let serialize = SerializationCallable::new(&self.serializer, include, exclude, extra);
            let v = if self.is_field_serializer {
                if let Some(model) = extra.model {
                    if self.info_arg {
                        let info = SerializationInfo::new(include, exclude, extra, self.is_field_serializer)?;
                        self.func.call1(py, (model, value, serialize, info))?
                    } else {
                        self.func.call1(py, (model, value, serialize))?
                    }
                } else {
                    return Err(PyRuntimeError::new_err("Function wrap serializer expected to be run inside the context of a model field but no model was found"));
                }
            } else if self.info_arg {
                let info = SerializationInfo::new(include, exclude, extra, self.is_field_serializer)?;
                self.func.call1(py, (value, serialize, info))?
            } else {
                self.func.call1(py, (value, serialize))?
            };
            Ok((true, v))
        } else {
            Ok((false, value.clone().unbind()))
        }
    }

    fn get_fallback_serializer(&self) -> &CombinedSerializer {
        self.serializer.as_ref()
    }

    fn retry_with_lax_check(&self) -> bool {
        self.serializer.retry_with_lax_check() || self.return_serializer.retry_with_lax_check()
    }
}

impl_py_gc_traverse!(FunctionWrapSerializer {
    serializer,
    func,
    return_serializer
});

function_type_serializer!(FunctionWrapSerializer);

#[pyclass(module = "pydantic_core._pydantic_core")]
#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) struct SerializationCallable {
    serializer: Arc<CombinedSerializer>,
    extra_owned: ExtraOwned,
    filter: AnyFilter,
    include: Option<PyObject>,
    exclude: Option<PyObject>,
}

impl SerializationCallable {
    pub fn new(
        serializer: &Arc<CombinedSerializer>,
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
    ) -> Self {
        Self {
            serializer: serializer.clone(),
            extra_owned: ExtraOwned::new(extra),
            filter: AnyFilter::new(),
            include: include.map(|v| v.clone().unbind()),
            exclude: exclude.map(|v| v.clone().unbind()),
        }
    }

    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if let Some(include) = &self.include {
            visit.call(include)?;
        }
        if let Some(exclude) = &self.exclude {
            visit.call(exclude)?;
        }
        if let Some(model) = &self.extra_owned.model {
            visit.call(model)?;
        }
        if let Some(fallback) = &self.extra_owned.fallback {
            visit.call(fallback)?;
        }
        if let Some(context) = &self.extra_owned.context {
            visit.call(context)?;
        }
        Ok(())
    }

    fn __clear__(&mut self) {
        self.include = None;
        self.exclude = None;
        self.extra_owned.model = None;
        self.extra_owned.fallback = None;
        self.extra_owned.context = None;
    }
}

#[pymethods]
impl SerializationCallable {
    #[pyo3(signature = (value, index_key=None))]
    fn __call__(
        &mut self,
        py: Python,
        value: &Bound<'_, PyAny>,
        index_key: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Option<PyObject>> {
        // NB wrap serializers have strong coupling to their inner type,
        // so use to_python_no_infer so that type inference can't apply
        // at this layer

        let include = self.include.as_ref().map(|o| o.bind(py));
        let exclude = self.exclude.as_ref().map(|o| o.bind(py));
        let extra = self.extra_owned.to_extra(py);

        if let Some(index_key) = index_key {
            let filter = if let Ok(index) = index_key.extract::<usize>() {
                self.filter.index_filter(index, include, exclude, None)?
            } else {
                self.filter.key_filter(index_key, include, exclude)?
            };
            if let Some((next_include, next_exclude)) = filter {
                let v =
                    self.serializer
                        .to_python_no_infer(value, next_include.as_ref(), next_exclude.as_ref(), &extra)?;
                extra.warnings.final_check(py)?;
                Ok(Some(v))
            } else {
                Err(PydanticOmit::new_err())
            }
        } else {
            let v = self.serializer.to_python_no_infer(value, include, exclude, &extra)?;
            extra.warnings.final_check(py)?;
            Ok(Some(v))
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "SerializationCallable(serializer={})",
            self.serializer.get_name()
        ))
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }
}

#[pyclass(module = "pydantic_core._pydantic_core")]
#[cfg_attr(debug_assertions, derive(Debug))]
struct SerializationInfo {
    #[pyo3(get)]
    include: Option<PyObject>,
    #[pyo3(get)]
    exclude: Option<PyObject>,
    #[pyo3(get)]
    context: Option<PyObject>,
    #[pyo3(get, name = "mode")]
    _mode: SerMode,
    #[pyo3(get)]
    by_alias: Option<bool>,
    #[pyo3(get)]
    exclude_unset: bool,
    #[pyo3(get)]
    exclude_defaults: bool,
    #[pyo3(get)]
    exclude_none: bool,
    #[pyo3(get)]
    round_trip: bool,
    field_name: Option<String>,
    #[pyo3(get)]
    serialize_as_any: bool,
}

impl SerializationInfo {
    fn new(
        include: Option<&Bound<'_, PyAny>>,
        exclude: Option<&Bound<'_, PyAny>>,
        extra: &Extra,
        is_field_serializer: bool,
    ) -> PyResult<Self> {
        if is_field_serializer {
            match extra.field_name {
                Some(field_name) => Ok(Self {
                    include: include.map(|i| i.clone().unbind()),
                    exclude: exclude.map(|e| e.clone().unbind()),
                    context: extra.context.map(|c| c.clone().unbind()),
                    _mode: extra.mode.clone(),
                    by_alias: extra.by_alias,
                    exclude_unset: extra.exclude_unset,
                    exclude_defaults: extra.exclude_defaults,
                    exclude_none: extra.exclude_none,
                    round_trip: extra.round_trip,
                    field_name: Some(field_name.to_string()),
                    serialize_as_any: extra.serialize_as_any,
                }),
                _ => Err(PyRuntimeError::new_err(
                    "Model field context expected for field serialization info but no model field was found",
                )),
            }
        } else {
            Ok(Self {
                include: include.map(|i| i.clone().unbind()),
                exclude: exclude.map(|e| e.clone().unbind()),
                context: extra.context.map(|c| c.clone().unbind()),
                _mode: extra.mode.clone(),
                by_alias: extra.by_alias,
                exclude_unset: extra.exclude_unset,
                exclude_defaults: extra.exclude_defaults,
                exclude_none: extra.exclude_none,
                round_trip: extra.round_trip,
                field_name: None,
                serialize_as_any: extra.serialize_as_any,
            })
        }
    }

    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        if let Some(include) = &self.include {
            visit.call(include)?;
        }
        if let Some(exclude) = &self.exclude {
            visit.call(exclude)?;
        }
        if let Some(context) = &self.context {
            visit.call(context)?;
        }
        Ok(())
    }

    fn __clear__(&mut self) {
        self.include = None;
        self.exclude = None;
        self.context = None;
    }
}

#[pymethods]
impl SerializationInfo {
    fn mode_is_json(&self) -> bool {
        self._mode.is_json()
    }

    #[getter]
    fn __dict__<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new(py);
        if let Some(ref include) = self.include {
            d.set_item("include", include)?;
        }
        if let Some(ref exclude) = self.exclude {
            d.set_item("exclude", exclude)?;
        }
        if let Some(ref context) = self.context {
            d.set_item("context", context)?;
        }
        d.set_item("mode", &self._mode)?;
        d.set_item("by_alias", self.by_alias)?;
        d.set_item("exclude_unset", self.exclude_unset)?;
        d.set_item("exclude_defaults", self.exclude_defaults)?;
        d.set_item("exclude_none", self.exclude_none)?;
        d.set_item("round_trip", self.round_trip)?;
        d.set_item("serialize_as_any", self.serialize_as_any)?;
        Ok(d)
    }

    fn __repr__(&self, py: Python) -> PyResult<String> {
        Ok(format!(
            "SerializationInfo(include={}, exclude={}, context={}, mode='{}', by_alias={}, exclude_unset={}, exclude_defaults={}, exclude_none={}, round_trip={}, serialize_as_any={})",
            match self.include {
                Some(ref include) => include.bind(py).repr()?.to_str()?.to_owned(),
                None => "None".to_owned(),
            },
            match self.exclude {
                Some(ref exclude) => exclude.bind(py).repr()?.to_str()?.to_owned(),
                None => "None".to_owned(),
            },
            match self.context {
                Some(ref context) => context.bind(py).repr()?.to_str()?.to_owned(),
                None => "None".to_owned(),
            },
            self._mode,
            py_bool(self.by_alias.unwrap_or(false)),
            py_bool(self.exclude_unset),
            py_bool(self.exclude_defaults),
            py_bool(self.exclude_none),
            py_bool(self.round_trip),
            py_bool(self.serialize_as_any),
        ))
    }

    fn __str__(&self, py: Python) -> PyResult<String> {
        self.__repr__(py)
    }
    #[getter]
    fn get_field_name<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyString>> {
        match self.field_name {
            Some(ref field_name) => Ok(PyString::new(py, field_name)),
            None => Err(PyAttributeError::new_err("No attribute named 'field_name'")),
        }
    }
}

fn py_bool(value: bool) -> &'static str {
    if value {
        "True"
    } else {
        "False"
    }
}
