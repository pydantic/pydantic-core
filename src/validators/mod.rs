use std::fmt::Debug;

use enum_dispatch::enum_dispatch;

use ahash::AHashSet;
use pyo3::exceptions::PyTypeError;
use pyo3::intern;
use pyo3::once_cell::GILOnceCell;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyByteArray, PyBytes, PyDict, PyList, PyString};

use crate::build_tools::{py_error, SchemaDict, SchemaError};
use crate::errors::{ErrorKind, ValError, ValLineError, ValResult, ValidationError};
use crate::input::{Input, JsonInput};
use crate::questions::{Answers, Question};
use crate::recursion_guard::RecursionGuard;

mod any;
mod arguments;
mod bool;
mod bytes;
mod call;
mod callable;
mod date;
mod datetime;
mod dict;
mod float;
mod frozenset;
mod function;
mod int;
mod is_instance;
mod list;
mod literal;
mod new_class;
mod none;
mod nullable;
mod recursive;
mod set;
mod string;
mod time;
mod timedelta;
mod tuple;
mod typed_dict;
mod union;
mod with_default;

#[pyclass(module = "pydantic_core._pydantic_core")]
#[derive(Debug, Clone)]
pub struct SchemaValidator {
    validator: CombinedValidator,
    slots: Vec<CombinedValidator>,
    schema: PyObject,
    title: PyObject,
}

#[pymethods]
impl SchemaValidator {
    #[new]
    pub fn py_new(py: Python, schema: &PyAny, config: Option<&PyDict>) -> PyResult<Self> {
        let self_schema = Self::get_self_schema(py);

        let schema_obj = self_schema
            .validator
            .validate(
                py,
                schema,
                &Extra::default(),
                &self_schema.slots,
                &mut RecursionGuard::default(),
            )
            .map_err(|e| SchemaError::from_val_error(py, e))?;
        let schema = schema_obj.as_ref(py);

        let mut used_refs = AHashSet::new();
        extract_used_refs(schema, &mut used_refs)?;
        let mut build_context = BuildContext::new(used_refs);

        let mut validator = build_validator(schema, config, &mut build_context)?;
        validator.complete(&build_context)?;
        let slots = build_context.into_slots()?;
        let title = validator.get_name().into_py(py);
        Ok(Self {
            validator,
            slots,
            schema: schema.into_py(py),
            title,
        })
    }

    pub fn __reduce__(&self, py: Python) -> PyResult<PyObject> {
        let args = (self.schema.as_ref(py),);
        let cls = Py::new(py, self.to_owned())?.getattr(py, "__class__")?;
        Ok((cls, args).into_py(py))
    }

    pub fn validate_python(
        &self,
        py: Python,
        input: &PyAny,
        strict: Option<bool>,
        context: Option<&PyAny>,
    ) -> PyResult<PyObject> {
        let r = self.validator.validate(
            py,
            input,
            &Extra::new(strict, context),
            &self.slots,
            &mut RecursionGuard::default(),
        );
        r.map_err(|e| self.prepare_validation_err(py, e))
    }

    pub fn isinstance_python(
        &self,
        py: Python,
        input: &PyAny,
        strict: Option<bool>,
        context: Option<&PyAny>,
    ) -> PyResult<bool> {
        match self.validator.validate(
            py,
            input,
            &Extra::new(strict, context),
            &self.slots,
            &mut RecursionGuard::default(),
        ) {
            Ok(_) => Ok(true),
            Err(ValError::InternalErr(err)) => Err(err),
            _ => Ok(false),
        }
    }

    pub fn validate_json(
        &self,
        py: Python,
        input: &PyAny,
        strict: Option<bool>,
        context: Option<&PyAny>,
    ) -> PyResult<PyObject> {
        match parse_json(input)? {
            Ok(input) => {
                let r = self.validator.validate(
                    py,
                    &input,
                    &Extra::new(strict, context),
                    &self.slots,
                    &mut RecursionGuard::default(),
                );
                r.map_err(|e| self.prepare_validation_err(py, e))
            }
            Err(e) => {
                let line_err = ValLineError::new(ErrorKind::InvalidJson { error: e.to_string() }, input);
                let err = ValError::LineErrors(vec![line_err]);
                Err(self.prepare_validation_err(py, err))
            }
        }
    }

    pub fn isinstance_json(
        &self,
        py: Python,
        input: &PyAny,
        strict: Option<bool>,
        context: Option<&PyAny>,
    ) -> PyResult<bool> {
        match parse_json(input)? {
            Ok(input) => {
                match self.validator.validate(
                    py,
                    &input,
                    &Extra::new(strict, context),
                    &self.slots,
                    &mut RecursionGuard::default(),
                ) {
                    Ok(_) => Ok(true),
                    Err(ValError::InternalErr(err)) => Err(err),
                    _ => Ok(false),
                }
            }
            Err(_) => Ok(false),
        }
    }

    pub fn validate_assignment(
        &self,
        py: Python,
        field: String,
        input: &PyAny,
        data: &PyDict,
        strict: Option<bool>,
        context: Option<&PyAny>,
    ) -> PyResult<PyObject> {
        let extra = Extra {
            data: Some(data),
            field: Some(field.as_str()),
            strict,
            context,
        };
        let r = self
            .validator
            .validate(py, input, &extra, &self.slots, &mut RecursionGuard::default());
        r.map_err(|e| self.prepare_validation_err(py, e))
    }

    pub fn __repr__(&self) -> String {
        format!(
            "SchemaValidator(name={:?}, validator={:#?})",
            self.validator.get_name(),
            self.validator
        )
    }
}

static SCHEMA_DEFINITION: GILOnceCell<SchemaValidator> = GILOnceCell::new();

impl SchemaValidator {
    fn get_self_schema(py: Python) -> &Self {
        SCHEMA_DEFINITION.get_or_init(py, || Self::build_self_schema(py).unwrap())
    }

    fn build_self_schema(py: Python) -> PyResult<Self> {
        let code = include_str!("../self_schema.py");
        let locals = PyDict::new(py);
        py.run(code, None, Some(locals))?;
        let self_schema: &PyDict = locals.get_as_req(intern!(py, "self_schema"))?;

        let mut used_refs = AHashSet::new();
        // NOTE: we don't call `extract_used_refs` for performance reasons, if more recursive references
        // are used, they would need to be manually added here.
        used_refs.insert("root-schema".to_string());
        let mut build_context = BuildContext::new(used_refs);

        let validator = match build_validator(self_schema, None, &mut build_context) {
            Ok(v) => v,
            Err(err) => return Err(SchemaError::new_err(format!("Error building self-schema:\n  {}", err))),
        };
        Ok(Self {
            validator,
            slots: build_context.into_slots()?,
            schema: py.None(),
            title: "Self Schema".into_py(py),
        })
    }

    fn prepare_validation_err(&self, py: Python, error: ValError) -> PyErr {
        ValidationError::from_val_error(py, self.title.clone_ref(py), error)
    }
}

fn parse_json(input: &PyAny) -> PyResult<serde_json::Result<JsonInput>> {
    if let Ok(py_bytes) = input.cast_as::<PyBytes>() {
        Ok(serde_json::from_slice(py_bytes.as_bytes()))
    } else if let Ok(py_str) = input.cast_as::<PyString>() {
        let str = py_str.to_str()?;
        Ok(serde_json::from_str(str))
    } else if let Ok(py_byte_array) = input.cast_as::<PyByteArray>() {
        Ok(serde_json::from_slice(unsafe { py_byte_array.as_bytes() }))
    } else {
        let input_type = input.get_type().name().unwrap_or("unknown");
        py_error!(PyTypeError; "JSON input should be str, bytes or bytearray, not {}", input_type)
    }
}

pub trait BuildValidator: Sized {
    const EXPECTED_TYPE: &'static str;

    /// Build a new validator from the schema, the return type is a trait to provide a way for validators
    /// to return other validators, see `string.rs`, `int.rs`, `float.rs` and `function.rs` for examples
    fn build(schema: &PyDict, config: Option<&PyDict>, build_context: &mut BuildContext)
        -> PyResult<CombinedValidator>;
}

/// Logic to create a particular validator, called in the `validator_match` macro, then in turn by `build_validator`
fn build_specific_validator<'a, T: BuildValidator>(
    val_type: &str,
    schema_dict: &'a PyDict,
    config: Option<&'a PyDict>,
    build_context: &mut BuildContext,
) -> PyResult<CombinedValidator> {
    let py = schema_dict.py();
    if let Some(schema_ref) = schema_dict.get_as::<String>(intern!(py, "ref"))? {
        // we only want to use a RecursiveContainerValidator if the ref is actually used,
        // this means refs can always be set without having an effect on the validator which is generated
        // unless it's used/referenced
        if build_context.ref_used(&schema_ref) {
            let answers = Answers::new(schema_dict)?;
            let slot_id = build_context.prepare_slot(schema_ref, answers.clone())?;
            let inner_val = T::build(schema_dict, config, build_context)?;
            let name = inner_val.get_name().to_string();
            build_context.complete_slot(slot_id, inner_val)?;
            return Ok(recursive::RecursiveRefValidator::from_id(slot_id, name, answers));
        }
    }

    T::build(schema_dict, config, build_context)
        .map_err(|err| SchemaError::new_err(format!("Error building \"{}\" validator:\n  {}", val_type, err)))
}

// macro to build the match statement for validator selection
macro_rules! validator_match {
    ($type:ident, $dict:ident, $config:ident, $build_context:ident, $($validator:path,)+) => {
        match $type {
            $(
                <$validator>::EXPECTED_TYPE => build_specific_validator::<$validator>($type, $dict, $config, $build_context),
            )+
            _ => {
                return py_error!(r#"Unknown schema type: "{}""#, $type)
            },
        }
    };
}

pub fn build_validator<'a>(
    schema: &'a PyAny,
    config: Option<&'a PyDict>,
    build_context: &mut BuildContext,
) -> PyResult<CombinedValidator> {
    let py = schema.py();
    let dict: &PyDict = match schema.cast_as() {
        Ok(s) => s,
        Err(_) => {
            let dict = PyDict::new(py);
            dict.set_item("type", schema)?;
            dict
        }
    };
    let type_: &str = dict.get_as_req(intern!(py, "type"))?;
    validator_match!(
        type_,
        dict,
        config,
        build_context,
        // typed dict e.g. heterogeneous dicts or simply a model
        typed_dict::TypedDictValidator,
        // unions
        union::UnionValidator,
        union::TaggedUnionValidator,
        // nullables
        nullable::NullableValidator,
        // model classes
        new_class::NewClassValidator,
        // strings
        string::StrValidator,
        // integers
        int::IntValidator,
        // boolean
        bool::BoolValidator,
        // floats
        float::FloatValidator,
        // tuples
        tuple::TupleBuilder,
        // list/arrays
        list::ListValidator,
        // sets - unique lists
        set::SetValidator,
        // dicts/objects (recursive)
        dict::DictValidator,
        // None/null
        none::NoneValidator,
        // functions - before, after, plain & wrap
        function::FunctionBuilder,
        // function call - validation around a function call
        call::CallValidator,
        // recursive (self-referencing) models
        recursive::RecursiveRefValidator,
        // literals
        literal::LiteralBuilder,
        // any
        any::AnyValidator,
        // bytes
        bytes::BytesValidator,
        // dates
        date::DateValidator,
        // times
        time::TimeValidator,
        // datetimes
        datetime::DateTimeValidator,
        // frozensets
        frozenset::FrozenSetValidator,
        // timedelta
        timedelta::TimeDeltaValidator,
        // introspection types
        is_instance::IsInstanceValidator,
        callable::CallableValidator,
        // arguments
        arguments::ArgumentsValidator,
        // default value
        with_default::WithDefaultValidator,
    )
}

/// More (mostly immutable) data to pass between validators, should probably be class `Context`,
/// but that would confuse it with context as per samuelcolvin/pydantic#1549
#[derive(Debug, Default)]
pub struct Extra<'a> {
    /// This is used as the `data` kwargs to validator functions, it also represents the current model
    /// data when validating assignment
    pub data: Option<&'a PyDict>,
    /// The field being assigned to when validating assignment
    pub field: Option<&'a str>,
    /// whether we're in strict or lax mode
    pub strict: Option<bool>,
    /// context used in validator functions
    pub context: Option<&'a PyAny>,
}

impl<'a> Extra<'a> {
    pub fn new(strict: Option<bool>, context: Option<&'a PyAny>) -> Self {
        Extra {
            strict,
            context,
            ..Default::default()
        }
    }
}

impl<'a> Extra<'a> {
    pub fn as_strict(&self) -> Self {
        Self {
            data: self.data,
            field: self.field,
            strict: Some(true),
            context: self.context,
        }
    }
}

#[derive(Debug, Clone)]
#[enum_dispatch]
pub enum CombinedValidator {
    // typed dict e.g. heterogeneous dicts or simply a model
    TypedDict(typed_dict::TypedDictValidator),
    // unions
    Union(union::UnionValidator),
    TaggedUnion(union::TaggedUnionValidator),
    // nullables
    Nullable(nullable::NullableValidator),
    // model classes
    ModelClass(new_class::NewClassValidator),
    // strings
    Str(string::StrValidator),
    StrConstrained(string::StrConstrainedValidator),
    // integers
    Int(int::IntValidator),
    ConstrainedInt(int::ConstrainedIntValidator),
    // booleans
    Bool(bool::BoolValidator),
    // floats
    Float(float::FloatValidator),
    ConstrainedFloat(float::ConstrainedFloatValidator),
    // lists
    List(list::ListValidator),
    // sets - unique lists
    Set(set::SetValidator),
    // tuples
    TuplePositional(tuple::TuplePositionalValidator),
    TupleVariable(tuple::TupleVariableValidator),
    // dicts/objects (recursive)
    Dict(dict::DictValidator),
    // None/null
    None(none::NoneValidator),
    // functions
    FunctionBefore(function::FunctionBeforeValidator),
    FunctionAfter(function::FunctionAfterValidator),
    FunctionPlain(function::FunctionPlainValidator),
    FunctionWrap(function::FunctionWrapValidator),
    // function call - validation around a function call
    FunctionCall(call::CallValidator),
    // recursive (self-referencing) models
    RecursiveRef(recursive::RecursiveRefValidator),
    // literals
    LiteralSingleString(literal::LiteralSingleStringValidator),
    LiteralSingleInt(literal::LiteralSingleIntValidator),
    LiteralMultipleStrings(literal::LiteralMultipleStringsValidator),
    LiteralMultipleInts(literal::LiteralMultipleIntsValidator),
    LiteralGeneral(literal::LiteralGeneralValidator),
    // any
    Any(any::AnyValidator),
    // bytes
    Bytes(bytes::BytesValidator),
    ConstrainedBytes(bytes::BytesConstrainedValidator),
    // dates
    Date(date::DateValidator),
    // times
    Time(time::TimeValidator),
    // datetimes
    Datetime(datetime::DateTimeValidator),
    // frozensets
    FrozenSet(frozenset::FrozenSetValidator),
    // timedelta
    Timedelta(timedelta::TimeDeltaValidator),
    // introspection types
    IsInstance(is_instance::IsInstanceValidator),
    Callable(callable::CallableValidator),
    // arguments
    Arguments(arguments::ArgumentsValidator),
    // default value
    WithDefault(with_default::WithDefaultValidator),
}

/// This trait must be implemented by all validators, it allows various validators to be accessed consistently,
/// validators defined in `build_validator` also need `EXPECTED_TYPE` as a const, but that can't be part of the trait
#[enum_dispatch(CombinedValidator)]
pub trait Validator: Send + Sync + Clone + Debug {
    /// Do the actual validation for this schema/type
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject>;

    /// `get_name` generally returns `Self::EXPECTED_TYPE` or some other clear identifier of the validator
    /// this is used in the error location in unions, and in the top level message in `ValidationError`
    fn get_name(&self) -> &str;

    /// allows validators to ask specific questions of sub-validators in a general way, could be extended
    /// to do more, validators which don't know the question and have sub-validators
    /// should return the result them in an `...iter().all(|v| v.ask(question))` way, ONLY
    /// if they return the value of the sub-validator, e.g. functions, unions
    fn ask(&self, _question: &Question) -> bool {
        false
    }

    /// this method must be implemented for any validator which holds references to other validators,
    /// it is used by `RecursiveRefValidator` to set its name
    fn complete(&mut self, _build_context: &BuildContext) -> PyResult<()> {
        Ok(())
    }
}

#[derive(Clone)]
struct Slot {
    slot_ref: String,
    op_validator: Option<CombinedValidator>,
    answers: Answers,
}

/// `BuildContext` is used to store extra information while building validators,
/// currently it just holds a vec "slots" which holds validators need to be accessed from multiple other validators
/// and therefore can't be owned by them directly.
#[derive(Default, Clone)]
pub struct BuildContext {
    used_refs: AHashSet<String>,
    slots: Vec<Slot>,
}

impl BuildContext {
    pub fn new(used_refs: AHashSet<String>) -> Self {
        Self {
            used_refs,
            ..Default::default()
        }
    }

    /// check if a ref is used elsewhere in the schema
    pub fn ref_used(&self, ref_: &str) -> bool {
        self.used_refs.contains(ref_)
    }

    /// First of two part process to add a new validator slot, we add the `slot_ref` to the array, but not the
    /// actual `validator`, we can't add the validator until it's build.
    /// We need the `id` to build the validator, hence this two-step process.
    pub fn prepare_slot(&mut self, slot_ref: String, answers: Answers) -> PyResult<usize> {
        let id = self.slots.len();
        let slot = Slot {
            slot_ref,
            op_validator: None,
            answers,
        };
        self.slots.push(slot);
        Ok(id)
    }

    /// Second part of adding a validator - we update the slot to include a validator
    pub fn complete_slot(&mut self, slot_id: usize, validator: CombinedValidator) -> PyResult<()> {
        match self.slots.get(slot_id) {
            Some(slot) => {
                self.slots[slot_id] = Slot {
                    slot_ref: slot.slot_ref.clone(),
                    op_validator: Some(validator),
                    answers: slot.answers.clone(),
                };
                Ok(())
            }
            None => py_error!("Slots Error: slot {} not found", slot_id),
        }
    }

    /// find a slot by `slot_ref` - iterate over the slots until we find a matching reference - return the index
    pub fn find_slot_id_answer(&self, slot_ref: &str) -> PyResult<(usize, Answers)> {
        let is_match = |slot: &Slot| slot.slot_ref == slot_ref;
        match self.slots.iter().position(is_match) {
            Some(id) => {
                let slot = self.slots.get(id).unwrap();
                Ok((id, slot.answers.clone()))
            }
            None => py_error!("Slots Error: ref '{}' not found", slot_ref),
        }
    }

    /// find a validator by `slot_id` - this used in `Validator.complete`, specifically `RecursiveRefValidator`
    /// to set its name
    pub fn find_validator(&self, slot_id: usize) -> PyResult<&CombinedValidator> {
        match self.slots.get(slot_id) {
            Some(slot) => match slot.op_validator {
                Some(ref validator) => Ok(validator),
                None => py_error!("Slots Error: slot {} not yet filled", slot_id),
            },
            None => py_error!("Slots Error: slot {} not found", slot_id),
        }
    }

    /// Move validators into a new vec which maintains the order of slots, `complete` is called on each validator
    /// at the same time.
    pub fn into_slots(self) -> PyResult<Vec<CombinedValidator>> {
        let self_clone = self.clone();
        self.slots
            .into_iter()
            .map(|slot| match slot.op_validator {
                Some(mut validator) => {
                    validator.complete(&self_clone)?;
                    Ok(validator)
                }
                None => py_error!("Slots Error: slot not yet filled"),
            })
            .collect()
    }
}

fn extract_used_refs(schema: &PyAny, refs: &mut AHashSet<String>) -> PyResult<()> {
    if let Ok(dict) = schema.cast_as::<PyDict>() {
        let py = schema.py();
        if matches!(dict.get_as(intern!(py, "type")), Ok(Some("recursive-ref"))) {
            refs.insert(dict.get_as_req(intern!(py, "schema_ref"))?);
        } else {
            for (_, value) in dict.iter() {
                extract_used_refs(value, refs)?;
            }
        }
    } else if let Ok(list) = schema.cast_as::<PyList>() {
        for item in list.iter() {
            extract_used_refs(item, refs)?;
        }
    }
    Ok(())
}
