use std::collections::hash_map::Entry;
use std::hash::BuildHasherDefault;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PySet, PyString};

use ahash::RandomState;
use nohash_hasher::{IntMap, IntSet, NoHashHasher};

use crate::build_tools::{is_strict, py_err, schema_or_config, schema_or_config_same, SchemaDict};
use crate::errors::{py_err_string, ErrorType, ValError, ValLineError, ValResult};
use crate::input::{
    AttributesGenericIterator, DictGenericIterator, GenericMapping, Input, JsonObjectGenericIterator,
    MappingGenericIterator,
};
use crate::lookup_key::LookupKey;
use crate::questions::Question;
use crate::recursion_guard::RecursionGuard;

use super::with_default::get_default;
use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct TypedDictField {
    pub name: String,
    pub lookup_key: LookupKey,
    pub name_pystring: Py<PyString>,
    pub required: bool,
    pub validator: CombinedValidator,
    pub frozen: bool,
}

impl TypedDictField {
    pub fn new(
        key: &PyAny,
        value: &PyAny,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
        total: bool,
        populate_by_name: bool,
    ) -> PyResult<Self> {
        let py = key.py();
        let field_info: &PyDict = value.cast_as()?;
        let field_name: &str = key.extract()?;

        let schema = field_info.get_as_req(intern!(py, "schema"))?;

        let validator = match build_validator(schema, config, build_context) {
            Ok(v) => v,
            Err(err) => return py_err!("Field \"{}\":\n  {}", field_name, err),
        };

        let required = match field_info.get_as::<bool>(intern!(py, "required"))? {
            Some(required) => {
                if required {
                    if let CombinedValidator::WithDefault(ref val) = validator {
                        if val.has_default() {
                            return py_err!("Field '{}': a required field cannot have a default value", field_name);
                        }
                    }
                }
                required
            }
            None => total,
        };

        if required {
            if let CombinedValidator::WithDefault(ref val) = validator {
                if val.omit_on_error() {
                    return py_err!(
                        "Field '{}': 'on_error = omit' cannot be set for required fields",
                        field_name
                    );
                }
            }
        }

        let lookup_key = match field_info.get_item(intern!(py, "alias")) {
            Some(alias) => {
                let alt_alias = if populate_by_name { Some(field_name) } else { None };
                LookupKey::from_py(py, alias, alt_alias)?
            }
            None => LookupKey::from_string(py, field_name),
        };

        Ok(Self {
            name: field_name.to_string(),
            lookup_key,
            name_pystring: PyString::intern(py, field_name).into(),
            validator,
            required,
            frozen: field_info.get_as::<bool>(intern!(py, "frozen"))?.unwrap_or(false),
        })
    }
}

pub type NHHBuilder<T> = BuildHasherDefault<NoHashHasher<T>>;

#[derive(Debug, Clone)]
pub struct FieldMap {
    map: IntMap<u64, Vec<usize>>,
    pub fields: Vec<TypedDictField>,
    hash_builder: RandomState,
}

impl FieldMap {
    pub fn new(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
        total: bool,
    ) -> PyResult<Self> {
        let py = schema.py();

        let populate_by_name = schema_or_config_same(schema, config, intern!(py, "populate_by_name"))?.unwrap_or(false);

        let fields_dict: &PyDict = schema.get_as_req(intern!(py, "fields"))?;
        let capacity = fields_dict.len();

        let mut map: IntMap<u64, Vec<usize>> = IntMap::with_capacity_and_hasher(capacity, NHHBuilder::default());
        let mut fields: Vec<TypedDictField> = Vec::with_capacity(capacity);
        let hash_builder = RandomState::new();

        for (key, value) in fields_dict.iter() {
            let field = TypedDictField::new(key, value, config, build_context, total, populate_by_name)?;
            let index = fields.len();
            for key in field.lookup_key.first_keys() {
                match map.entry(hash_builder.hash_one(key)) {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().push(index);
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(vec![index]);
                    }
                };
            }
            fields.push(field);
        }

        Ok(Self {
            map,
            fields,
            hash_builder,
        })
    }

    pub fn get<'a>(&'a self, key: &'_ str) -> Option<FieldMatch<'a>> {
        let hash = self.hash_builder.hash_one(key);
        self.map.get(&hash).map(|field_ids| FieldMatch::new(field_ids, self))
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
}

pub struct FieldMatch<'a> {
    index: usize,
    field_ids: &'a [usize],
    fields: &'a [TypedDictField],
}

impl<'a> FieldMatch<'a> {
    fn new(field_ids: &'a [usize], field_map: &'a FieldMap) -> Self {
        Self {
            index: 0,
            field_ids,
            fields: &field_map.fields,
        }
    }
}

impl<'a> Iterator for FieldMatch<'a> {
    type Item = (usize, &'a TypedDictField);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.field_ids.len() {
            let fields_index = self.field_ids[self.index];
            let field = &self.fields[fields_index];
            self.index += 1;
            Some((fields_index, field))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypedDictValidator {
    fields_map: FieldMap,
    check_extra: bool,
    forbid_extra: bool,
    extra_validator: Option<Box<CombinedValidator>>,
    strict: bool,
    from_attributes: bool,
    return_fields_set: bool,
}

impl BuildValidator for TypedDictValidator {
    const EXPECTED_TYPE: &'static str = "typed-dict";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let strict = is_strict(schema, config)?;

        let extra_behavior = schema_or_config::<&str>(
            schema,
            config,
            intern!(py, "extra_behavior"),
            intern!(py, "typed_dict_extra_behavior"),
        )?;

        let (check_extra, forbid_extra) = match extra_behavior {
            Some(s) => match s {
                "allow" => (true, false),
                "ignore" => (false, false),
                "forbid" => (true, true),
                _ => return py_err!(r#"Invalid extra_behavior: "{}""#, s),
            },
            None => (false, false),
        };

        let extra_validator = match schema.get_item(intern!(py, "extra_validator")) {
            Some(v) => {
                if check_extra && !forbid_extra {
                    Some(Box::new(build_validator(v, config, build_context)?))
                } else {
                    return py_err!("extra_validator can only be used if extra_behavior=allow");
                }
            }
            None => None,
        };

        let total =
            schema_or_config(schema, config, intern!(py, "total"), intern!(py, "typed_dict_total"))?.unwrap_or(true);

        Ok(Self {
            fields_map: FieldMap::new(schema, config, build_context, total)?,
            check_extra,
            forbid_extra,
            extra_validator,
            strict,
            from_attributes: schema_or_config_same(schema, config, intern!(py, "from_attributes"))?.unwrap_or(false),
            return_fields_set: schema.get_as(intern!(py, "return_fields_set"))?.unwrap_or(false),
        }
        .into())
    }
}

impl Validator for TypedDictValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        if let Some(field) = extra.field {
            // we're validating assignment, completely different logic
            return self.validate_assignment(py, field, input, extra, slots, recursion_guard);
        }
        let strict = extra.strict.unwrap_or(self.strict);
        let dict = input.validate_typed_dict(strict, self.from_attributes)?;

        let fields_len = self.fields_map.len();
        let mut output_vec: Vec<(usize, Py<PyString>, PyObject)> = Vec::with_capacity(fields_len);
        let mut errors: Vec<ValLineError> = Vec::with_capacity(fields_len);
        let mut fields_set_vec: Option<Vec<Py<PyString>>> = match self.return_fields_set {
            true => Some(Vec::with_capacity(fields_len)),
            false => None,
        };

        let mut used_fields: IntSet<usize> = IntSet::with_capacity_and_hasher(fields_len, NHHBuilder::default());

        let extra = Extra {
            // data: Some(output_dict),
            data: None,
            field: None,
            strict: extra.strict,
            context: extra.context,
        };

        macro_rules! process {
            ($dict:ident, $get_method:ident, $iter:ty) => {{
                for item_result in <$iter>::new($dict)? {
                    let (raw_key, value) = item_result?;
                    let either_str = match raw_key.validate_str(strict) {
                        Ok(k) => k,
                        Err(ValError::LineErrors(line_errors)) => {
                            for err in line_errors {
                                errors.push(
                                    err.with_outer_location(raw_key.as_loc_item())
                                        .with_type(ErrorType::InvalidKey),
                                );
                            }
                            continue;
                        }
                        Err(err) => return Err(err),
                    };
                    let key_str: &str = &either_str.as_cow()?;

                    if let Some(fields_matchs) = self.fields_map.get(key_str) {
                        for (field_index, field) in fields_matchs {
                            // this field is used whatever happens as we don't want to go over it again later
                            used_fields.insert(field_index);
                            match field.lookup_key.$get_method(key_str, value) {
                                Ok(Some(value)) => {
                                    // this field is used whether or not validation passes, since we don't
                                    // want to check this field again
                                    used_fields.insert(field_index);
                                    match field
                                        .validator
                                        .validate(py, value, &extra, slots, recursion_guard)
                                    {
                                        Ok(value) => {
                                            output_vec.push((field_index, field.name_pystring.clone_ref(py), value));
                                            if let Some(ref mut fs) = fields_set_vec {
                                                fs.push(field.name_pystring.clone_ref(py));
                                            }
                                        }
                                        Err(ValError::Omit) => (),
                                        Err(ValError::LineErrors(line_errors)) => {
                                            for err in line_errors {
                                                errors.push(err.with_outer_location(field.name.clone().into()));
                                            }
                                        }
                                        Err(err) => return Err(err),
                                    }
                                }
                                Ok(None) => {
                                    if let Some(value) = get_default(py, &field.validator)? {
                                        let default_value = value.as_ref().clone_ref(py);
                                        output_vec.push((
                                            field_index,
                                            field.name_pystring.clone_ref(py),
                                            default_value,
                                        ));
                                    } else if field.required {
                                        errors.push(ValLineError::new_with_loc(
                                            ErrorType::Missing,
                                            input,
                                            field.name.clone(),
                                        ));
                                    }
                                }
                                Err(err) => {
                                    errors.push(ValLineError::new_with_loc(
                                        ErrorType::GetAttributeError {
                                            error: py_err_string(py, err),
                                        },
                                        input,
                                        field.name.clone(),
                                    ));
                                }
                            };
                        }
                    } else if self.forbid_extra {
                        errors.push(ValLineError::new_with_loc(
                            ErrorType::ExtraForbidden,
                            value,
                            raw_key.as_loc_item(),
                        ));
                    } else if self.check_extra {
                        let py_key = either_str.as_py_string(py);
                        if let Some(ref validator) = self.extra_validator {
                            match validator.validate(py, value, &extra, slots, recursion_guard) {
                                Ok(value) => {
                                    output_vec.push((fields_len, py_key.into_py(py), value));
                                    if let Some(ref mut fs) = fields_set_vec {
                                        fs.push(py_key.into_py(py));
                                    }
                                }
                                Err(ValError::LineErrors(line_errors)) => {
                                    for err in line_errors {
                                        errors.push(err.with_outer_location(raw_key.as_loc_item()));
                                    }
                                }
                                Err(err) => return Err(err),
                            }
                        } else {
                            output_vec.push((fields_len, py_key.into_py(py), value.to_object(py)));
                            if let Some(ref mut fs) = fields_set_vec {
                                fs.push(py_key.into_py(py));
                            }
                        }
                    }
                }
                // iterate over any missing fields and either add them to errors or set them to default
                if used_fields.len() != fields_len {
                    for (field_index, field) in self.fields_map.fields.iter().enumerate() {
                        if !used_fields.contains(&field_index) {
                            if let Some(value) = get_default(py, &field.validator)? {
                                let default_value = value.as_ref().clone_ref(py);
                                output_vec.push((field_index, field.name_pystring.clone_ref(py), default_value));
                            } else if field.required {
                                errors.push(ValLineError::new_with_loc(
                                    ErrorType::Missing,
                                    input,
                                    field.name.clone(),
                                ));
                            }
                        }
                    }
                }
            }};
        }
        match dict {
            GenericMapping::PyDict(d) => process!(d, py_mapping_pair, DictGenericIterator),
            GenericMapping::PyMapping(d) => process!(d, py_mapping_pair, MappingGenericIterator),
            GenericMapping::PyGetAttr(d) => process!(d, py_get_attr_pair, AttributesGenericIterator),
            GenericMapping::JsonObject(d) => process!(d, json_get_pair, JsonObjectGenericIterator),
        }

        if !errors.is_empty() {
            return Err(ValError::LineErrors(errors));
        }

        output_vec.sort_by(|a, b| a.0.cmp(&b.0));
        let output_dict = PyDict::new(py);
        for (_, key, value) in output_vec {
            output_dict.set_item(key, value)?;
        }

        if let Some(fs) = fields_set_vec {
            let fields_set = PySet::new(py, &fs)?;
            Ok((output_dict, fields_set).to_object(py))
        } else {
            Ok(output_dict.to_object(py))
        }
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }

    fn ask(&self, question: &Question) -> bool {
        match question {
            Question::ReturnFieldsSet => self.return_fields_set,
        }
    }

    fn complete(&mut self, build_context: &BuildContext) -> PyResult<()> {
        self.fields_map
            .fields
            .iter_mut()
            .try_for_each(|f| f.validator.complete(build_context))
    }
}

impl TypedDictValidator {
    fn validate_assignment<'s, 'data>(
        &'s self,
        py: Python<'data>,
        field: &str,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject>
    where
        'data: 's,
    {
        // TODO probably we should set location on errors here
        let data = match extra.data {
            Some(data) => data,
            None => unreachable!(),
        };

        let prepare_tuple = |output: PyObject| {
            data.set_item(field, output)?;
            if self.return_fields_set {
                let fields_set = PySet::new(py, &[field])?;
                Ok((data, fields_set).to_object(py))
            } else {
                Ok(data.to_object(py))
            }
        };

        let prepare_result = |result: ValResult<'data, PyObject>| match result {
            Ok(output) => prepare_tuple(output),
            Err(ValError::LineErrors(line_errors)) => {
                let errors = line_errors
                    .into_iter()
                    .map(|e| e.with_outer_location(field.to_string().into()))
                    .collect();
                Err(ValError::LineErrors(errors))
            }
            Err(err) => Err(err),
        };

        if let Some(field) = self.fields_map.fields.iter().find(|f| f.name == field) {
            if field.frozen {
                Err(ValError::new_with_loc(ErrorType::Frozen, input, field.name.to_string()))
            } else {
                prepare_result(field.validator.validate(py, input, extra, slots, recursion_guard))
            }
        } else if self.check_extra && !self.forbid_extra {
            // this is the "allow" case of extra_behavior
            match self.extra_validator {
                Some(ref validator) => prepare_result(validator.validate(py, input, extra, slots, recursion_guard)),
                None => prepare_tuple(input.to_object(py)),
            }
        } else {
            // otherwise we raise an error:
            // - with forbid this is obvious
            // - with ignore the model should never be overloaded, so an error is the clearest option
            Err(ValError::new_with_loc(
                ErrorType::ExtraForbidden,
                input,
                field.to_string(),
            ))
        }
    }
}
