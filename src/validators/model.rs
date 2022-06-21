use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PySet, PyString};

use crate::build_tools::{py_error, SchemaDict};
use crate::errors::{as_internal, err_val_error, val_line_error, ErrorKind, ValError, ValLineError, ValResult};
use crate::input::{GenericMapping, Input, JsonInput, JsonObject, ToLocItem};

use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
struct ModelField {
    name: String,
    key: FieldKey,
    dict_key: Py<PyString>,
    default: Option<PyObject>,
    validator: CombinedValidator,
}

#[derive(Debug, Clone)]
pub struct ModelValidator {
    name: String,
    fields: Vec<ModelField>,
    extra_behavior: ExtraBehavior,
    extra_validator: Option<Box<CombinedValidator>>,
}

impl BuildValidator for ModelValidator {
    const EXPECTED_TYPE: &'static str = "model";

    fn build(
        schema: &PyDict,
        _config: Option<&PyDict>,
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        // models ignore the parent config and always use the config from this model
        let config: Option<&PyDict> = schema.get_as("config")?;

        let extra_behavior = ExtraBehavior::from_config(config)?;
        let extra_validator = match extra_behavior {
            ExtraBehavior::Allow => match schema.get_item("extra_validator") {
                Some(v) => Some(Box::new(build_validator(v, config, build_context)?.0)),
                None => None,
            },
            _ => None,
        };

        let name: String = schema.get_as("name")?.unwrap_or_else(|| "Model".to_string());
        let fields_dict: &PyDict = match schema.get_as("fields")? {
            Some(fields) => fields,
            None => {
                // allow an empty model, is this is a good idea?
                return Ok(Self {
                    name,
                    fields: vec![],
                    extra_behavior,
                    extra_validator,
                }
                .into());
            }
        };
        let mut fields: Vec<ModelField> = Vec::with_capacity(fields_dict.len());

        let py = schema.py();
        for (key, value) in fields_dict.iter() {
            let field_infos: &PyDict = value.cast_as()?;
            let schema: &PyAny = field_infos.get_as_req("schema")?;

            let validator = match build_validator(schema, config, build_context) {
                Ok((v, _)) => v,
                Err(err) => return py_error!("Key \"{}\":\n  {}", key, err),
            };

            let key_str = key.to_string();
            let key = match field_infos.get_item("alias") {
                Some(alias) => {
                    if let Ok(string_alias) = alias.extract::<String>() {
                        FieldKey::Choice((key_str.clone(), string_alias))
                    } else if let Ok(list) = alias.cast_as::<PyList>() {
                        let locs = list
                            .iter()
                            .map(|item| FieldKeyLoc::from_py(item))
                            .collect::<PyResult<Vec<FieldKeyLoc>>>()?;
                        FieldKey::PathChoices(vec![locs])
                    } else {
                        return py_error!(r#""alias" must be a string or list of strings & ints"#);
                    }
                }
                None => FieldKey::Simple(key_str.clone()),
            };
            fields.push(ModelField {
                name: key_str.clone(),
                key,
                dict_key: PyString::intern(py, &key_str).into(),
                validator,
                default: field_infos.get_as("default")?,
            });
        }
        Ok(Self {
            name,
            fields,
            extra_behavior,
            extra_validator,
        }
        .into())
    }
}

impl Validator for ModelValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject> {
        if let Some(field) = extra.field {
            // we're validating assignment, completely different logic
            return self.validate_assignment(py, field, input, extra, slots);
        }

        // TODO we shouldn't always use try_instance=true here
        let dict = input.lax_dict(true)?;
        let output_dict = PyDict::new(py);
        let mut errors: Vec<ValLineError> = Vec::new();
        let fields_set = PySet::empty(py).map_err(as_internal)?;

        let extra = Extra {
            data: Some(output_dict),
            field: None,
        };

        macro_rules! process {
            ($dict:ident, $get_method:ident) => {{
                for field in &self.fields {
                    let py_key: &PyString = field.dict_key.as_ref(py);
                    if let Some(value) = $get_method($dict, &field.key) {
                        match field.validator.validate(py, value, &extra, slots) {
                            Ok(value) => output_dict.set_item(py_key, value).map_err(as_internal)?,
                            Err(ValError::LineErrors(line_errors)) => {
                                for err in line_errors {
                                    errors.push(err.with_prefix_location(field.name.to_loc()));
                                }
                            }
                            Err(err) => return Err(err),
                        }
                        fields_set.add(py_key).map_err(as_internal)?;
                    } else if let Some(ref default) = field.default {
                        output_dict
                            .set_item(py_key, default.as_ref(py))
                            .map_err(as_internal)?;
                    } else {
                        errors.push(val_line_error!(
                            input_value = input.as_error_value(),
                            kind = ErrorKind::Missing,
                            location = vec![field.name.to_loc()]
                        ));
                    }
                }

                let (check_extra, forbid) = match self.extra_behavior {
                    ExtraBehavior::Ignore => (false, false),
                    ExtraBehavior::Allow => (true, false),
                    ExtraBehavior::Forbid => (true, true),
                };
                if check_extra {
                    for (raw_key, value) in $dict.iter() {
                        // TODO use strict_str here if the model is strict
                        let key: String = match raw_key.lax_str() {
                            Ok(k) => k.as_raw().map_err(as_internal)?,
                            Err(ValError::LineErrors(line_errors)) => {
                                for err in line_errors {
                                    errors.push(err.with_prefix_location(raw_key.to_loc()));
                                }
                                continue;
                            }
                            Err(err) => return Err(err),
                        };
                        let py_key = PyString::new(py, &key);
                        if fields_set.contains(py_key).map_err(as_internal)? {
                            continue;
                        }
                        fields_set.add(py_key).map_err(as_internal)?;

                        if forbid {
                            errors.push(val_line_error!(
                                input_value = input.as_error_value(),
                                kind = ErrorKind::ExtraForbidden,
                                location = vec![key.to_loc()]
                            ));
                        } else if let Some(ref validator) = self.extra_validator {
                            match validator.validate(py, value, &extra, slots) {
                                Ok(value) => output_dict.set_item(py_key, value).map_err(as_internal)?,
                                Err(ValError::LineErrors(line_errors)) => {
                                    for err in line_errors {
                                        errors.push(err.with_prefix_location(key.to_loc()));
                                    }
                                }
                                Err(err) => return Err(err),
                            }
                        } else {
                            output_dict
                                .set_item(&key, value.to_object(py))
                                .map_err(as_internal)?;
                        }
                    }
                }
            }};
        }
        match dict {
            GenericMapping::PyDict(d) => process!(d, pydict_get),
            GenericMapping::JsonObject(d) => process!(d, jsonobject_get),
        }

        if errors.is_empty() {
            Ok((output_dict, fields_set).to_object(py))
        } else {
            Err(ValError::LineErrors(errors))
        }
    }

    fn get_name(&self, _py: Python) -> String {
        self.name.clone()
    }
}

impl ModelValidator {
    fn validate_assignment<'s, 'data>(
        &'s self,
        py: Python<'data>,
        field: &str,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
    ) -> ValResult<'data, PyObject>
    where
        'data: 's,
    {
        // TODO probably we should set location on errors here
        let data = match extra.data {
            Some(data) => data,
            None => panic!("data is required when validating assignment"),
        };

        let prepare_tuple = |output: PyObject| {
            data.set_item(field, output).map_err(as_internal)?;
            let fields_set = PySet::new(py, &[field]).map_err(as_internal)?;
            Ok((data, fields_set).to_object(py))
        };

        let prepare_result = |result: ValResult<'data, PyObject>| match result {
            Ok(output) => prepare_tuple(output),
            Err(ValError::LineErrors(line_errors)) => {
                let errors = line_errors
                    .into_iter()
                    .map(|e| e.with_prefix_location(field.to_loc()))
                    .collect();
                Err(ValError::LineErrors(errors))
            }
            Err(err) => Err(err),
        };

        if let Some(field) = self.fields.iter().find(|f| f.name == field) {
            prepare_result(field.validator.validate(py, input, extra, slots))
        } else {
            match self.extra_behavior {
                // with allow we either want to set the value
                ExtraBehavior::Allow => match self.extra_validator {
                    Some(ref validator) => prepare_result(validator.validate(py, input, extra, slots)),
                    None => prepare_tuple(input.to_object(py)),
                },
                // otherwise we raise an error:
                // - with forbid this is obvious
                // - with ignore the model should never be overloaded, so an error is the clearest option
                _ => {
                    let loc = vec![field.to_loc()];
                    err_val_error!(
                        input_value = input.as_error_value(),
                        location = loc,
                        kind = ErrorKind::ExtraForbidden
                    )
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
enum ExtraBehavior {
    Allow,
    Ignore,
    Forbid,
}

impl ExtraBehavior {
    pub fn from_config(config: Option<&PyDict>) -> PyResult<Self> {
        match config {
            Some(dict) => {
                let b: Option<String> = dict.get_as("extra")?;
                match b {
                    Some(s) => match s.as_str() {
                        "allow" => Ok(ExtraBehavior::Allow),
                        "ignore" => Ok(ExtraBehavior::Ignore),
                        "forbid" => Ok(ExtraBehavior::Forbid),
                        _ => py_error!(r#"Invalid extra_behavior: "{}""#, s),
                    },
                    None => Ok(ExtraBehavior::Ignore),
                }
            }
            None => Ok(ExtraBehavior::Ignore),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FieldKeyLoc {
    // string type key, used to get items from a dict
    StrKey(String),
    // integer key, used to get items from a list, tuple OR a dict with int keys
    IntKey(usize),
}

impl ToPyObject for FieldKeyLoc {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            Self::StrKey(val) => val.to_object(py),
            Self::IntKey(val) => val.to_object(py),
        }
    }
}

impl FieldKeyLoc {
    pub fn from_py(obj: &PyAny) -> PyResult<Self> {
        if let Ok(str_key) = obj.extract::<String>() {
            Ok(FieldKeyLoc::StrKey(str_key))
        } else if let Ok(int_key) = obj.extract::<usize>() {
            Ok(FieldKeyLoc::IntKey(int_key))
        } else {
            py_error!("if alias is a list, it must contains only strings and int")
        }
    }

    pub fn json_get<'a>(&self, any_json: &'a JsonInput) -> Option<&'a JsonInput> {
        match self {
            Self::StrKey(key) => match any_json {
                JsonInput::Object(v_obj) => v_obj.get(key),
                _ => None,
            },
            Self::IntKey(index) => match any_json {
                JsonInput::Array(v_array) => v_array.get(*index),
                _ => None,
            },
        }
    }

    pub fn json_obj_get<'a>(&self, json_obj: &'a JsonObject) -> Option<&'a JsonInput> {
        match self {
            Self::StrKey(key) => json_obj.get(key),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FieldKey {
    Simple(String),
    Choice((String, String)),
    PathChoices(Vec<Vec<FieldKeyLoc>>),
}

fn pydict_get<'data, 's>(dict: &'data PyDict, field_key: &'s FieldKey) -> Option<&'data PyAny> {
    match field_key {
        FieldKey::Simple(key) => dict.get_item(key),
        FieldKey::Choice((key1, key2)) => {
            match dict.get_item(key1) {
                Some(v) => Some(v),
                None => dict.get_item(key2),
            }
        }
        FieldKey::PathChoices(path_choices) => {
            'paths_loop: for path in path_choices {
                let mut v: &PyAny = dict;

                for loc in path {
                    // blindly try getitem on v since no better logic is realistic
                    // TODO we could perhaps try getattr for StrKey depending on try_instance
                    v = match v.get_item(loc) {
                        Ok(v_next) => v_next,
                        // key/index not found, try next path
                        Err(_) => continue 'paths_loop,
                    }
                }

                // Successfully found an item, return it
                return Some(v);
            }
            // got to the end of path_choices, without a match, return None
            None
        }
    }
}

fn jsonobject_get<'data, 's>(dict: &'data JsonObject, field_key: &'s FieldKey) -> Option<&'data JsonInput> {
    match field_key {
        FieldKey::Simple(key) => dict.get(key),
        FieldKey::Choice((key1, key2)) => {
            match dict.get(key1) {
                Some(v) => Some(v),
                None => dict.get(key2),
            }
        }
        FieldKey::PathChoices(path_choices) => {
            'paths_loop: for path in path_choices {
                let mut path_iter = path.iter();

                let mut v: &JsonInput;

                // first step is different from the rest as we already know dict is JsonObject
                if let Some(loc) = path_iter.next() {
                    v = match loc.json_obj_get(&dict) {
                        Some(v) => v,
                        None => continue 'paths_loop,
                    }
                } else {
                    continue 'paths_loop;
                }

                for loc in path_iter {
                    if let Some(next_v) = loc.json_get(v) {
                        v = next_v;
                    } else {
                        continue 'paths_loop;
                    }
                }

                // Successfully found an item, return it
                return Some(v);
            }
            // got to the end of path_choices, without a match, return None
            None
        }
    }
}
