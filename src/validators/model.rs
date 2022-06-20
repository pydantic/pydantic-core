use pyo3::prelude::*;
use pyo3::types::{PyDict, PySet, PyString};

use crate::build_tools::{py_error, SchemaDict};
use crate::errors::{as_internal, err_val_error, val_line_error, ErrorKind, ValError, ValLineError, ValResult};
use crate::input::{GenericMapping, Input, ToLocItem};

use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
struct ModelField {
    name: String,
    // alias: Option<String>,
    dict_key: Py<PyString>,
    default: Option<PyObject>,
    validator_id: usize,
}

#[derive(Debug, Clone)]
pub struct ModelValidator {
    name: String,
    fields: [Option<ModelField>; 32],
    // fields: Vec<Option<ModelField>>,
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

        // let mut fields: [Option<ModelField>; 100] = unsafe {
        //     let mut arr: [Option<ModelField>; 100] = std::mem::uninitialized();
        //     for item in &mut arr[..] {
        //         std::ptr::write(item, None);
        //     }
        //     arr
        // };
        let mut fields: [Option<ModelField>; 32]  = Default::default();
        let name: String = schema.get_as("name")?.unwrap_or_else(|| "Model".to_string());
        let fields_dict: &PyDict = match schema.get_as("fields")? {
            Some(fields) => fields,
            None => {
                // allow an empty model, is this is a good idea?
                return Ok(Self {
                    name,
                    fields,
                    extra_behavior,
                    extra_validator,
                }
                .into());
            }
        };
        // let mut fields: Vec<Option<ModelField>> = Vec::with_capacity(fields_dict.len());

        let py = schema.py();
        for (index, (key, value)) in fields_dict.iter().enumerate() {
            let (validator, field_dict) = match build_validator(value, config, build_context) {
                Ok(v) => v,
                Err(err) => return py_error!("Key \"{}\":\n  {}", key, err),
            };

            assert!(index < 32);
            let key_str = key.to_string();
            fields[index] = Some(ModelField {
                name: key_str.clone(),
                // alias: field_dict.get_as("alias"),
                dict_key: PyString::intern(py, &key_str).into(),
                validator_id: build_context.add_existing_validator(validator),
                default: field_dict.get_as("default")?,
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
                for field_opt in &self.fields {
                    match field_opt {
                        Some(field) => {
                            let py_key: &PyString = field.dict_key.as_ref(py);
                            if let Some(value) = $dict.$get_method(&field.name) {
                                let validator = unsafe {slots.get_unchecked(field.validator_id) };
                                match validator.validate(py, value, &extra, slots) {
                                    Ok(value) => output_dict.set_item(py_key, value).map_err(as_internal)?,
                                    Err(ValError::LineErrors(line_errors)) => {
                                        let loc = vec![field.name.to_loc()];
                                        for err in line_errors {
                                            errors.push(err.with_prefix_location(&loc));
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
                        },
                        None => break,
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
                            Ok(k) => k,
                            Err(ValError::LineErrors(line_errors)) => {
                                let loc = vec![raw_key.to_loc()];
                                for err in line_errors {
                                    errors.push(err.with_prefix_location(&loc));
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
                        let loc = vec![key.to_loc()];

                        if forbid {
                            errors.push(val_line_error!(
                                input_value = input.as_error_value(),
                                kind = ErrorKind::ExtraForbidden,
                                location = loc
                            ));
                        } else if let Some(ref validator) = self.extra_validator {
                            match validator.validate(py, value, &extra, slots) {
                                Ok(value) => output_dict.set_item(py_key, value).map_err(as_internal)?,
                                Err(ValError::LineErrors(line_errors)) => {
                                    for err in line_errors {
                                        errors.push(err.with_prefix_location(&loc));
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
            GenericMapping::PyDict(d) => process!(d, get_item),
            GenericMapping::JsonObject(d) => process!(d, get),
        };

        if errors.is_empty() {
            Ok((output_dict, fields_set).to_object(py))
        } else {
            Err(ValError::LineErrors(errors))
        }
    }

    fn get_name<'data>(&self, _py: Python, _slots: &'data [CombinedValidator]) -> String {
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
                let loc = vec![field.to_loc()];
                let errors = line_errors.into_iter().map(|e| e.with_prefix_location(&loc)).collect();
                Err(ValError::LineErrors(errors))
            }
            Err(err) => Err(err),
        };

        let find_field = |op_f: &Option<ModelField>| {
            match op_f {
                Some(f) if f.name == field => Some(f.validator_id),
                _ => None,
            }
        };

        if let Some(validator_id) = self.fields.iter().find_map(find_field) {
            let validator = unsafe {slots.get_unchecked(validator_id) };
            prepare_result(validator.validate(py, input, extra, slots))
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
