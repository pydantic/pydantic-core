use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::sync::Arc;

use ahash::AHashMap;
use jiter::JsonArray;
use jiter::JsonObject;
use jiter::JsonValue;
use pyo3::exceptions::PyKeyError;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PySet, PyString, PyType};

use ahash::AHashSet;
use pyo3::IntoPyObjectExt;

use crate::build_tools::py_schema_err;
use crate::build_tools::{is_strict, schema_or_config_same, ExtraBehavior};
use crate::errors::LocItem;
use crate::errors::{ErrorType, ErrorTypeDefaults, ValError, ValLineError, ValResult};
use crate::input::ConsumeIterator;
use crate::input::{BorrowInput, Input, ValidatedDict, ValidationMatch};
use crate::lookup_key::LookupKey;
use crate::lookup_key::LookupPath;
use crate::lookup_key::PathItem;
use crate::tools::new_py_string;
use crate::tools::SchemaDict;

use super::{build_validator, BuildValidator, CombinedValidator, DefinitionsBuilder, ValidationState, Validator};

#[derive(Debug)]
struct Field {
    name: String,
    lookup_key: LookupKey,
    name_py: Py<PyString>,
    validator: CombinedValidator,
    frozen: bool,
}

impl_py_gc_traverse!(Field { validator });

#[derive(Debug)]
pub struct ModelFieldsValidator {
    fields: Vec<Field>,
    model_name: String,
    extra_behavior: ExtraBehavior,
    extras_validator: Option<Box<CombinedValidator>>,
    strict: bool,
    from_attributes: bool,
    loc_by_alias: bool,
    lookup: LookupMap,
}

impl BuildValidator for ModelFieldsValidator {
    const EXPECTED_TYPE: &'static str = "model-fields";

    fn build(
        schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();
        let strict = is_strict(schema, config)?;

        let from_attributes = schema_or_config_same(schema, config, intern!(py, "from_attributes"))?.unwrap_or(false);
        let populate_by_name = schema_or_config_same(schema, config, intern!(py, "populate_by_name"))?.unwrap_or(false);

        let extra_behavior = ExtraBehavior::from_schema_or_config(py, schema, config, ExtraBehavior::Ignore)?;

        let extras_validator = match (schema.get_item(intern!(py, "extras_schema"))?, &extra_behavior) {
            (Some(v), ExtraBehavior::Allow) => Some(Box::new(build_validator(&v, config, definitions)?)),
            (Some(_), _) => return py_schema_err!("extras_schema can only be used if extra_behavior=allow"),
            (_, _) => None,
        };
        let model_name: String = schema
            .get_as(intern!(py, "model_name"))?
            .unwrap_or_else(|| "Model".to_string());

        let fields_dict: Bound<'_, PyDict> = schema.get_as_req(intern!(py, "fields"))?;
        let mut fields: Vec<Field> = Vec::with_capacity(fields_dict.len());

        for (key, value) in fields_dict {
            let field_info = value.downcast::<PyDict>()?;
            let field_name_py: Bound<'_, PyString> = key.extract()?;
            let field_name = field_name_py.to_str()?;

            let schema = field_info.get_as_req(intern!(py, "schema"))?;

            let validator = match build_validator(&schema, config, definitions) {
                Ok(v) => v,
                Err(err) => return py_schema_err!("Field \"{}\":\n  {}", field_name, err),
            };

            let lookup_key = match field_info.get_item(intern!(py, "validation_alias"))? {
                Some(alias) => {
                    let alt_alias = if populate_by_name { Some(field_name) } else { None };
                    LookupKey::from_py(py, &alias, alt_alias)?
                }
                None => LookupKey::from_string(py, field_name),
            };

            fields.push(Field {
                name: field_name.to_string(),
                lookup_key,
                name_py: field_name_py.into(),
                validator,
                frozen: field_info.get_as::<bool>(intern!(py, "frozen"))?.unwrap_or(false),
            });
        }

        let mut map = AHashMap::new();

        fn add_field_to_map<K: Eq + Hash>(map: &mut AHashMap<K, LookupValue>, key: K, field_index: usize) {
            match map.entry(key) {
                Entry::Occupied(mut entry) => match entry.get_mut() {
                    &mut LookupValue::Field(i) => {
                        entry.insert(LookupValue::Complex {
                            fields: vec![i, field_index],
                            lookup_map: LookupMap {
                                map: AHashMap::new(),
                                list: AHashMap::new(),
                            },
                        });
                    }
                    LookupValue::Complex { fields, .. } => {
                        fields.push(field_index);
                    }
                },
                Entry::Vacant(entry) => {
                    entry.insert(LookupValue::Field(field_index));
                }
            }
        }

        fn add_path_to_map(map: &mut AHashMap<String, LookupValue>, path: &LookupPath, field_index: usize) {
            if path.rest().is_empty() {
                // terminal value
                add_field_to_map(map, path.first_key().to_owned(), field_index);
                return;
            }

            let mut nested_map = match map.entry(path.first_key().to_owned()) {
                Entry::Occupied(mut entry) => {
                    let entry = entry.into_mut();
                    match entry {
                        &mut LookupValue::Field(i) => {
                            *entry = LookupValue::Complex {
                                fields: vec![i],
                                lookup_map: LookupMap {
                                    map: AHashMap::new(),
                                    list: AHashMap::new(),
                                },
                            };
                            match entry {
                                LookupValue::Complex {
                                    lookup_map: ref mut nested_map,
                                    ..
                                } => nested_map,
                                _ => unreachable!(),
                            }
                        }
                        LookupValue::Complex {
                            lookup_map: ref mut nested_map,
                            ..
                        } => nested_map,
                    }
                }
                Entry::Vacant(entry) => {
                    let LookupValue::Complex {
                        lookup_map: ref mut nested_map,
                        ..
                    } = entry.insert(LookupValue::Complex {
                        fields: Vec::new(),
                        lookup_map: LookupMap {
                            map: AHashMap::new(),
                            list: AHashMap::new(),
                        },
                    })
                    else {
                        unreachable!()
                    };
                    nested_map
                }
            };

            let mut nested_map = nested_map;
            let mut path_iter = path.rest().iter();

            let mut current = path_iter.next().expect("rest is non-empty");

            while let Some(next) = path_iter.next() {
                nested_map = match current {
                    PathItem::S(s) => {
                        let str_key = s.key.to_owned();
                        match nested_map.map.entry(str_key) {
                            Entry::Occupied(entry) => {
                                let entry = entry.into_mut();
                                match entry {
                                    &mut LookupValue::Field(i) => {
                                        *entry = LookupValue::Complex {
                                            fields: vec![i],
                                            lookup_map: LookupMap {
                                                map: AHashMap::new(),
                                                list: AHashMap::new(),
                                            },
                                        };
                                        let LookupValue::Complex {
                                            lookup_map: ref mut nested_map,
                                            ..
                                        } = entry
                                        else {
                                            unreachable!()
                                        };
                                        nested_map
                                    }
                                    LookupValue::Complex {
                                        lookup_map: ref mut nested_map,
                                        ..
                                    } => nested_map,
                                }
                            }
                            Entry::Vacant(entry) => {
                                let LookupValue::Complex {
                                    lookup_map: ref mut nested_map,
                                    ..
                                } = entry.insert(LookupValue::Complex {
                                    fields: vec![],
                                    lookup_map: LookupMap {
                                        map: AHashMap::new(),
                                        list: AHashMap::new(),
                                    },
                                })
                                else {
                                    unreachable!()
                                };
                                nested_map
                            }
                        }
                    }
                    PathItem::Pos(i) => match nested_map.list.entry(*i as i64) {
                        Entry::Occupied(entry) => {
                            let entry = entry.into_mut();
                            match entry {
                                &mut LookupValue::Field(i) => {
                                    *entry = LookupValue::Complex {
                                        fields: vec![i],
                                        lookup_map: LookupMap {
                                            map: AHashMap::new(),
                                            list: AHashMap::new(),
                                        },
                                    };
                                    let LookupValue::Complex {
                                        lookup_map: ref mut nested_map,
                                        ..
                                    } = entry
                                    else {
                                        unreachable!()
                                    };
                                    nested_map
                                }
                                LookupValue::Complex {
                                    lookup_map: ref mut nested_map,
                                    ..
                                } => nested_map,
                            }
                        }
                        Entry::Vacant(entry) => {
                            let LookupValue::Complex {
                                lookup_map: ref mut nested_map,
                                ..
                            } = entry.insert(LookupValue::Complex {
                                fields: vec![],
                                lookup_map: LookupMap {
                                    map: AHashMap::new(),
                                    list: AHashMap::new(),
                                },
                            })
                            else {
                                unreachable!()
                            };
                            nested_map
                        }
                    },
                    // FIXME: handle integer cases
                    PathItem::Neg(i) => match nested_map.list.entry(-(*i as i64)) {
                        Entry::Occupied(entry) => {
                            let entry = entry.into_mut();
                            match entry {
                                &mut LookupValue::Field(i) => {
                                    *entry = LookupValue::Complex {
                                        fields: vec![i],
                                        lookup_map: LookupMap {
                                            map: AHashMap::new(),
                                            list: AHashMap::new(),
                                        },
                                    };
                                    let LookupValue::Complex {
                                        lookup_map: ref mut nested_map,
                                        ..
                                    } = entry
                                    else {
                                        unreachable!()
                                    };
                                    nested_map
                                }
                                LookupValue::Complex {
                                    lookup_map: ref mut nested_map,
                                    ..
                                } => nested_map,
                            }
                        }
                        Entry::Vacant(entry) => {
                            let LookupValue::Complex {
                                lookup_map: ref mut nested_map,
                                ..
                            } = entry.insert(LookupValue::Complex {
                                fields: vec![],
                                lookup_map: LookupMap {
                                    map: AHashMap::new(),
                                    list: AHashMap::new(),
                                },
                            })
                            else {
                                unreachable!()
                            };
                            nested_map
                        }
                    },
                };

                current = next;
            }

            // now have a terminal value
            match current {
                PathItem::S(s) => {
                    add_field_to_map(&mut nested_map.map, s.key.to_owned(), field_index);
                }
                PathItem::Pos(i) => {
                    add_field_to_map(&mut nested_map.list, *i as i64, field_index);
                }
                PathItem::Neg(i) => {
                    add_field_to_map(&mut nested_map.list, -(*i as i64), field_index);
                }
            }
        }

        for (i, field) in fields.iter().enumerate() {
            match &field.lookup_key {
                LookupKey::Simple(path) => {
                    // should be a single string key
                    debug_assert!(path.rest().is_empty());
                    add_field_to_map(&mut map, path.first_key().to_owned(), i);
                }
                LookupKey::Choice { path1, path2 } => {
                    // two choices of single string keys
                    debug_assert!(path1.rest().is_empty());
                    debug_assert!(path2.rest().is_empty());
                    add_field_to_map(&mut map, path1.first_key().to_owned(), i);
                    add_field_to_map(&mut map, path2.first_key().to_owned(), i);
                }
                LookupKey::PathChoices(paths) => {
                    for path in paths {
                        add_path_to_map(&mut map, path, i);
                    }
                }
            }
        }

        Ok(Self {
            fields,
            model_name,
            extra_behavior,
            extras_validator,
            strict,
            from_attributes,
            loc_by_alias: config.get_as(intern!(py, "loc_by_alias"))?.unwrap_or(true),
            lookup: LookupMap {
                map,
                list: AHashMap::new(),
            },
        }
        .into())
    }
}

impl_py_gc_traverse!(ModelFieldsValidator {
    fields,
    extras_validator
});

impl Validator for ModelFieldsValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        // this validator does not yet support partial validation, disable it to avoid incorrect results
        state.allow_partial = false.into();

        let strict = state.strict_or(self.strict);
        let from_attributes = state.extra().from_attributes.unwrap_or(self.from_attributes);

        let (model_dict, mut model_extra_dict_op, fields_set) = if let Some(json_input) = input.as_json() {
            let JsonValue::Object(json_object) = json_input else {
                return Err(ValError::new(
                    ErrorType::ModelType {
                        context: None,
                        class_name: self.model_name.clone(),
                    },
                    input,
                ));
            };
            self.validate_json_by_iteration(py, json_input, json_object, state)?
        } else {
            // we convert the DictType error to a ModelType error
            let dict = match input.validate_model_fields(strict, from_attributes) {
                Ok(d) => d,
                Err(ValError::LineErrors(errors)) => {
                    let errors: Vec<ValLineError> = errors
                        .into_iter()
                        .map(|e| match e.error_type {
                            ErrorType::DictType { .. } => {
                                let mut e = e;
                                e.error_type = ErrorType::ModelType {
                                    class_name: self.model_name.clone(),
                                    context: None,
                                };
                                e
                            }
                            _ => e,
                        })
                        .collect();
                    return Err(ValError::LineErrors(errors));
                }
                Err(err) => return Err(err),
            };
            self.validate_by_get_item(py, input, dict, state)?
        };
        state.add_fields_set(fields_set.len());

        // if we have extra=allow, but we didn't create a dict because we were validating
        // from attributes, set it now so __pydantic_extra__ is always a dict if extra=allow
        if matches!(self.extra_behavior, ExtraBehavior::Allow) && model_extra_dict_op.is_none() {
            model_extra_dict_op = Some(PyDict::new(py));
        };

        Ok((model_dict, model_extra_dict_op, fields_set).into_py_any(py)?)
    }

    fn validate_assignment<'py>(
        &self,
        py: Python<'py>,
        obj: &Bound<'py, PyAny>,
        field_name: &str,
        field_value: &Bound<'py, PyAny>,
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        let dict = obj.downcast::<PyDict>()?;

        let get_updated_dict = |output: &Bound<'py, PyAny>| {
            dict.set_item(field_name, output)?;
            Ok(dict)
        };

        let prepare_result = |result: ValResult<PyObject>| match result {
            Ok(output) => get_updated_dict(&output.into_bound(py)),
            Err(ValError::LineErrors(line_errors)) => {
                let errors = line_errors
                    .into_iter()
                    .map(|e| e.with_outer_location(field_name))
                    .collect();
                Err(ValError::LineErrors(errors))
            }
            Err(err) => Err(err),
        };

        // by using dict but removing the field in question, we match V1 behaviour
        let data_dict = dict.copy()?;
        if let Err(err) = data_dict.del_item(field_name) {
            // KeyError is fine here as the field might not be in the dict
            if !err.get_type(py).is(&PyType::new::<PyKeyError>(py)) {
                return Err(err.into());
            }
        }

        let new_data = {
            let state = &mut state.rebind_extra(move |extra| extra.data = Some(data_dict));

            if let Some(field) = self.fields.iter().find(|f| f.name == field_name) {
                if field.frozen {
                    return Err(ValError::new_with_loc(
                        ErrorTypeDefaults::FrozenField,
                        field_value,
                        field.name.to_string(),
                    ));
                }

                prepare_result(field.validator.validate(py, field_value, state))?
            } else {
                // Handle extra (unknown) field
                // We partially use the extra_behavior for initialization / validation
                // to determine how to handle assignment
                // For models / typed dicts we forbid assigning extra attributes
                // unless the user explicitly set extra_behavior to 'allow'
                match self.extra_behavior {
                    ExtraBehavior::Allow => match self.extras_validator {
                        Some(ref validator) => prepare_result(validator.validate(py, field_value, state))?,
                        None => get_updated_dict(field_value)?,
                    },
                    ExtraBehavior::Forbid | ExtraBehavior::Ignore => {
                        return Err(ValError::new_with_loc(
                            ErrorType::NoSuchAttribute {
                                attribute: field_name.to_string(),
                                context: None,
                            },
                            field_value,
                            field_name.to_string(),
                        ))
                    }
                }
            }
        };

        let new_extra = match &self.extra_behavior {
            ExtraBehavior::Allow => {
                let non_extra_data = PyDict::new(py);
                self.fields.iter().try_for_each(|f| -> PyResult<()> {
                    let Some(popped_value) = new_data.get_item(&f.name)? else {
                        // field not present in __dict__ for some reason; let the rest of the
                        // validation pipeline handle it later
                        return Ok(());
                    };
                    new_data.del_item(&f.name)?;
                    non_extra_data.set_item(&f.name, popped_value)?;
                    Ok(())
                })?;
                let new_extra = new_data.copy()?;
                new_data.clear();
                new_data.update(non_extra_data.as_mapping())?;
                new_extra.into()
            }
            _ => py.None(),
        };

        let fields_set = PySet::new(py, &[field_name.to_string()])?;
        Ok((new_data, new_extra, fields_set).into_py_any(py)?)
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}

type ValidatedModelFields<'py> = (Bound<'py, PyDict>, Option<Bound<'py, PyDict>>, Bound<'py, PySet>);

impl ModelFieldsValidator {
    fn validate_by_get_item<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        dict: impl ValidatedDict<'py>,
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<ValidatedModelFields<'py>> {
        let model_dict = PyDict::new(py);
        let mut model_extra_dict_op: Option<Bound<PyDict>> = None;
        let mut errors: Vec<ValLineError> = Vec::with_capacity(self.fields.len());
        let fields_set = PySet::empty(py)?;

        // we only care about which keys have been used if we're iterating over the object for extra after
        // the first pass
        let mut used_keys: Option<AHashSet<&str>> =
            if self.extra_behavior == ExtraBehavior::Ignore || dict.is_py_get_attr() {
                None
            } else {
                Some(AHashSet::with_capacity(self.fields.len()))
            };

        {
            let state = &mut state.rebind_extra(|extra| extra.data = Some(model_dict.clone()));

            for field in &self.fields {
                let op_key_value = match dict.get_item(&field.lookup_key) {
                    Ok(v) => v,
                    Err(ValError::LineErrors(line_errors)) => {
                        for err in line_errors {
                            errors.push(err.with_outer_location(&field.name));
                        }
                        continue;
                    }
                    Err(err) => return Err(err),
                };
                if let Some((lookup_path, value)) = op_key_value {
                    if let Some(ref mut used_keys) = used_keys {
                        // key is "used" whether or not validation passes, since we want to skip this key in
                        // extra logic either way
                        used_keys.insert(lookup_path.first_key());
                    }
                    match field.validator.validate(py, value.borrow_input(), state) {
                        Ok(value) => {
                            model_dict.set_item(&field.name_py, value)?;
                            fields_set.add(&field.name_py)?;
                        }
                        Err(ValError::Omit) => continue,
                        Err(ValError::LineErrors(line_errors)) => {
                            for err in line_errors {
                                errors.push(lookup_path.apply_error_loc(err, self.loc_by_alias, &field.name));
                            }
                        }
                        Err(err) => return Err(err),
                    }
                    continue;
                }

                match field.validator.default_value(py, Some(field.name.as_str()), state) {
                    Ok(Some(value)) => {
                        // Default value exists, and passed validation if required
                        model_dict.set_item(&field.name_py, value)?;
                    }
                    Ok(None) => {
                        // This means there was no default value
                        errors.push(field.lookup_key.error(
                            ErrorTypeDefaults::Missing,
                            input,
                            self.loc_by_alias,
                            &field.name,
                        ));
                    }
                    Err(ValError::Omit) => continue,
                    Err(ValError::LineErrors(line_errors)) => {
                        for err in line_errors {
                            // Note: this will always use the field name even if there is an alias
                            // However, we don't mind so much because this error can only happen if the
                            // default value fails validation, which is arguably a developer error.
                            // We could try to "fix" this in the future if desired.
                            errors.push(err);
                        }
                    }
                    Err(err) => return Err(err),
                }
            }
        }

        if let Some(used_keys) = used_keys {
            struct ValidateToModelExtra<'a, 's, 'py> {
                py: Python<'py>,
                used_keys: AHashSet<&'a str>,
                errors: &'a mut Vec<ValLineError>,
                fields_set: &'a Bound<'py, PySet>,
                extra_behavior: ExtraBehavior,
                extras_validator: Option<&'a CombinedValidator>,
                state: &'a mut ValidationState<'s, 'py>,
            }

            impl<'py, Key, Value> ConsumeIterator<ValResult<(Key, Value)>> for ValidateToModelExtra<'_, '_, 'py>
            where
                Key: BorrowInput<'py> + Clone + Into<LocItem>,
                Value: BorrowInput<'py>,
            {
                type Output = ValResult<Bound<'py, PyDict>>;
                fn consume_iterator(
                    self,
                    iterator: impl Iterator<Item = ValResult<(Key, Value)>>,
                ) -> ValResult<Bound<'py, PyDict>> {
                    let model_extra_dict = PyDict::new(self.py);
                    for item_result in iterator {
                        let (raw_key, value) = item_result?;
                        let either_str = match raw_key
                            .borrow_input()
                            .validate_str(true, false)
                            .map(ValidationMatch::into_inner)
                        {
                            Ok(k) => k,
                            Err(ValError::LineErrors(line_errors)) => {
                                for err in line_errors {
                                    self.errors.push(
                                        err.with_outer_location(raw_key.clone())
                                            .with_type(ErrorTypeDefaults::InvalidKey),
                                    );
                                }
                                continue;
                            }
                            Err(err) => return Err(err),
                        };
                        let cow = either_str.as_cow()?;
                        if self.used_keys.contains(cow.as_ref()) {
                            continue;
                        }

                        let value = value.borrow_input();
                        // Unknown / extra field
                        match self.extra_behavior {
                            ExtraBehavior::Forbid => {
                                self.errors.push(ValLineError::new_with_loc(
                                    ErrorTypeDefaults::ExtraForbidden,
                                    value,
                                    raw_key.clone(),
                                ));
                            }
                            ExtraBehavior::Ignore => {}
                            ExtraBehavior::Allow => {
                                let py_key = either_str.as_py_string(self.py, self.state.cache_str());
                                if let Some(validator) = self.extras_validator {
                                    match validator.validate(self.py, value, self.state) {
                                        Ok(value) => {
                                            model_extra_dict.set_item(&py_key, value)?;
                                            self.fields_set.add(py_key)?;
                                        }
                                        Err(ValError::LineErrors(line_errors)) => {
                                            for err in line_errors {
                                                self.errors.push(err.with_outer_location(raw_key.clone()));
                                            }
                                        }
                                        Err(err) => return Err(err),
                                    }
                                } else {
                                    model_extra_dict.set_item(&py_key, value.to_object(self.py)?)?;
                                    self.fields_set.add(py_key)?;
                                };
                            }
                        }
                    }
                    Ok(model_extra_dict)
                }
            }

            let model_extra_dict = dict.iterate(ValidateToModelExtra {
                py,
                used_keys,
                errors: &mut errors,
                fields_set: &fields_set,
                extra_behavior: self.extra_behavior,
                extras_validator: self.extras_validator.as_deref(),
                state,
            })??;

            if matches!(self.extra_behavior, ExtraBehavior::Allow) {
                model_extra_dict_op = Some(model_extra_dict);
            }
        }

        if !errors.is_empty() {
            return Err(ValError::LineErrors(errors));
        }

        Ok((model_dict, model_extra_dict_op, fields_set))
    }

    fn validate_json_by_iteration<'py>(
        &self,
        py: Python<'py>,
        json_input: &JsonValue<'_>,
        json_object: &JsonObject<'_>,
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<ValidatedModelFields<'py>> {
        // expect json_input and json_object to be the same thing, just projected
        debug_assert!(matches!(&json_input, JsonValue::Object(j) if Arc::ptr_eq(j, json_object)));

        let model_dict = PyDict::new(py);
        let mut model_extra_dict_op: Option<Bound<PyDict>> = None;
        let mut field_results: Vec<Option<Result<PyObject, ValError>>> = (0..self.fields.len()).map(|_| None).collect();
        let mut errors: Vec<ValLineError> = Vec::new();
        let fields_set = PySet::empty(py)?;

        fn consume_json_array<'py>(
            py: Python<'py>,
            fields: &[Field],
            field_results: &mut [Option<Result<PyObject, ValError>>],
            array_lookup: &AHashMap<i64, LookupValue>,
            json_array: &JsonArray<'_>,
            state: &mut ValidationState<'_, 'py>,
        ) -> ValResult<()> {
            for (list_item, value) in array_lookup {
                let index = if *list_item < 0 {
                    list_item + json_array.len() as i64
                } else {
                    *list_item
                };
                if let Some(json_value) = json_array.get(index as usize) {
                    match value {
                        &LookupValue::Field(i) => {
                            field_results[i] = Some(fields[i].validator.validate(py, json_value, state));
                        }
                        LookupValue::Complex {
                            fields: complex_lookup_fields,
                            lookup_map,
                        } => perform_complex_lookup(
                            py,
                            fields,
                            field_results,
                            complex_lookup_fields,
                            lookup_map,
                            json_value,
                            state,
                        )?,
                    }
                }
            }
            Ok(())
        }

        fn perform_complex_lookup<'py>(
            py: Python<'py>,
            fields: &[Field],
            field_results: &mut [Option<Result<PyObject, ValError>>],
            complex_lookup_fields: &[usize],
            complex_lookup_map: &LookupMap,
            json_value: &JsonValue<'_>,
            state: &mut ValidationState<'_, 'py>,
        ) -> ValResult<()> {
            // this is a possibly recursive lookup with some complicated alias logic,
            // not much we can do except recurse
            for &i in complex_lookup_fields {
                field_results[i] = Some(fields[i].validator.validate(py, json_value, state));
            }
            if !complex_lookup_map.map.is_empty() {
                if let JsonValue::Object(nested_object) = json_value {
                    for (key, value) in &**nested_object {
                        if let Some(lookup_value) = complex_lookup_map.map.get(key.as_ref()) {
                            match lookup_value {
                                &LookupValue::Field(i) => {
                                    field_results[i] = Some(fields[i].validator.validate(py, value, state));
                                }
                                LookupValue::Complex {
                                    fields: complex_lookup_fields,
                                    lookup_map,
                                } => {
                                    perform_complex_lookup(
                                        py,
                                        fields,
                                        field_results,
                                        complex_lookup_fields,
                                        lookup_map,
                                        value,
                                        state,
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
            if !complex_lookup_map.list.is_empty() {
                if let JsonValue::Array(nested_array) = json_value {
                    consume_json_array(py, fields, field_results, &complex_lookup_map.list, nested_array, state)?;
                }
            }
            Ok(())
        }

        let model_extra_dict = PyDict::new(py);
        for (key, value) in &**json_object {
            let key = key.as_ref();
            if let Some(lookup_value) = self.lookup.map.get(key) {
                match lookup_value {
                    &LookupValue::Field(i) => {
                        field_results[i] = Some(self.fields[i].validator.validate(py, value, state));
                    }
                    LookupValue::Complex { fields, lookup_map } => {
                        perform_complex_lookup(py, &self.fields, &mut field_results, fields, lookup_map, value, state)?;
                    }
                }
                continue;
            }

            // Unknown / extra field - we only care about these at the top level
            match self.extra_behavior {
                ExtraBehavior::Forbid => {
                    errors.push(ValLineError::new_with_loc(
                        ErrorTypeDefaults::ExtraForbidden,
                        value,
                        key,
                    ));
                }
                ExtraBehavior::Ignore => {}
                ExtraBehavior::Allow => {
                    let py_key: Bound<'_, PyString> = new_py_string(py, key, state.cache_str());
                    if let Some(validator) = &self.extras_validator {
                        match validator.validate(py, value, state) {
                            Ok(value) => {
                                model_extra_dict.set_item(&py_key, value)?;
                                fields_set.add(py_key)?;
                            }
                            Err(ValError::LineErrors(line_errors)) => {
                                for err in line_errors {
                                    errors.push(err.with_outer_location(key));
                                }
                            }
                            Err(err) => return Err(err),
                        }
                    } else {
                        model_extra_dict.set_item(&py_key, value)?;
                        fields_set.add(py_key)?;
                    };
                }
            }
        }

        // now that we've iterated over all the keys, we can set the values in the model
        // dict, and try to set defaults for any missing fields

        for (field, field_result) in std::iter::zip(&self.fields, field_results) {
            let field_value = if let Some(validation_result) = field_result {
                match validation_result {
                    Ok(value) => {
                        fields_set.add(&field.name_py)?;
                        value
                    }
                    Err(ValError::Omit) => continue,
                    Err(ValError::LineErrors(line_errors)) => {
                        for err in line_errors {
                            // FIXME this should use the lookup path which the result was found at
                            errors.push(err.with_outer_location(&field.name));
                        }
                        continue;
                    }
                    Err(err) => return Err(err),
                }
            } else {
                match field.validator.default_value(py, Some(field.name.as_str()), state) {
                    Ok(Some(default_value)) => default_value,
                    Ok(None) => {
                        errors.push(field.lookup_key.error(
                            ErrorTypeDefaults::Missing,
                            json_input,
                            self.loc_by_alias,
                            &field.name,
                        ));
                        continue;
                    }
                    Err(ValError::Omit) => continue,
                    Err(ValError::LineErrors(line_errors)) => {
                        for err in line_errors {
                            // Note: this will always use the field name even if there is an alias
                            // However, we don't mind so much because this error can only happen if the
                            // default value fails validation, which is arguably a developer error.
                            // We could try to "fix" this in the future if desired.
                            errors.push(err);
                        }
                        continue;
                    }
                    Err(err) => return Err(err),
                }
            };

            model_dict.set_item(&field.name_py, field_value)?;
        }

        if matches!(self.extra_behavior, ExtraBehavior::Allow) {
            model_extra_dict_op = Some(model_extra_dict);
        }

        if !errors.is_empty() {
            return Err(ValError::LineErrors(errors));
        }

        Ok((model_dict, model_extra_dict_op, fields_set))
    }
}

#[derive(Debug)]
enum LookupValue {
    /// This lookup hits an actual field
    Field(usize),
    /// This lookup might applicable to multiple fields
    Complex {
        /// All fields which wanted _exactly_ this key
        fields: Vec<usize>,
        /// Fields which use this key as path prefix
        lookup_map: LookupMap,
    },
}

#[derive(Debug)]
struct LookupMap {
    map: AHashMap<String, LookupValue>,
    list: AHashMap<i64, LookupValue>,
}
