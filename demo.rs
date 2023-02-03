#[enum_dispatch(CombinedValidator)]
pub trait Validator {
    const EXPECTED_TYPE: &'static str;

    fn build(schema: &PyDict, config: Option<&PyDict>) -> PyResult<CombinedValidator>;

    fn validate(&self, input: &impl Input, extra: &Extra) -> ValResult<PyObject>;
}

#[enum_dispatch]
pub enum CombinedValidator {
    Int(IntValidator),
    Str(StrValidator),
    TypedDict(TypedDictValidator),
    Union(UnionValidator),
    TaggedUnion(TaggedUnionValidator),
    Nullable(NullableValidator),
    // ... and 43 more
}

pub fn choose_validator<'a>(schema: &'a PyDict, config: Option<&'a PyDict>) -> PyResult<CombinedValidator> {
    let schema_type: &str = schema.get_as_req("type")?;
    // really this is a clever macro to avoid the duplication
    match schema_type {
        IntValidator::EXPECTED_TYPE => IntValidator::build(schema, config),
        StrValidator::EXPECTED_TYPE => StrValidator::build(schema, config),
        TypedDictValidator::EXPECTED_TYPE => TypedDictValidator::build(schema, config),
        UnionValidator::EXPECTED_TYPE => UnionValidator::build(schema, config),
        TaggedUnionValidator::EXPECTED_TYPE => TaggedUnionValidator::build(schema, config),
        NullableValidator::EXPECTED_TYPE => NullableValidator::build(schema, config),
        // ... and 43 more
    }
}

#[pyclass]
pub struct SchemaValidator {
    validator: CombinedValidator,
}

#[pymethods]
impl SchemaValidator {
    #[new]
    pub fn py_new(schema: &PyDict, config: Option<&PyDict>) -> PyResult<Self> {
        let validator = choose_validator(schema, config)?;
        Ok(SchemaValidator { validator })
    }

    pub fn validate_python(&self, input: &PyAny, strict: Option<bool>) -> PyResult<PyObject> {
        self.validator.validate(input, &Extra::new(strict))
    }

    pub fn validate_json(
        &self,
        input_string: &PyString,
        strict: Option<bool>,
    ) -> PyResult<PyObject> {
        let input = parse_string(input_string)?;
        self.validator.validate(&input, &Extra::new(strict))
    }
}
