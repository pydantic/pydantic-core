use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString, PyType};

use crate::build_tools::py_schema_err;
use crate::build_tools::{is_strict, schema_or_config, ExtraBehavior};
use crate::errors::LocItem;
use crate::errors::{ErrorTypeDefaults, ValError, ValLineError, ValResult};
use crate::input::BorrowInput;
use crate::input::ConsumeIterator;
use crate::input::ValidationMatch;
use crate::input::{Input, ValidatedDict};
use crate::lookup_key::LookupKeyCollection;
use crate::tools::SchemaDict;
use ahash::AHashSet;
use jiter::PartialMode;

use super::{build_validator, BuildValidator, CombinedValidator, DefinitionsBuilder, ValidationState, Validator};

#[derive(Debug)]
struct TypedDictField {
    name: String,
    lookup_key_collection: LookupKeyCollection,
    name_py: Py<PyString>,
    required: bool,
    validator: CombinedValidator,
}

impl_py_gc_traverse!(TypedDictField { validator });

#[derive(Debug)]
pub struct TypedDictValidator {
    fields: Vec<TypedDictField>,
    extra_behavior: ExtraBehavior,
    extras_validator: Option<Box<CombinedValidator>>,
    strict: bool,
    loc_by_alias: bool,
    validate_by_alias: Option<bool>,
    validate_by_name: Option<bool>,
    cls_name: Option<String>,
}

impl BuildValidator for TypedDictValidator {
    const EXPECTED_TYPE: &'static str = "typed-dict";

    fn build(
        schema: &Bound<'_, PyDict>,
        _config: Option<&Bound<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();

        // typed dicts ignore the parent config and always use the config from this TypedDict
        let config = schema.get_as(intern!(py, "config"))?;
        let config = config.as_ref();

        let strict = is_strict(schema, config)?;

        let total =
            schema_or_config(schema, config, intern!(py, "total"), intern!(py, "typed_dict_total"))?.unwrap_or(true);

        let extra_behavior = ExtraBehavior::from_schema_or_config(py, schema, config, ExtraBehavior::Ignore)?;

        let extras_validator = match (schema.get_item(intern!(py, "extras_schema"))?, &extra_behavior) {
            (Some(v), ExtraBehavior::Allow) => Some(Box::new(build_validator(&v, config, definitions)?)),
            (Some(_), _) => return py_schema_err!("extras_schema can only be used if extra_behavior=allow"),
            (_, _) => None,
        };

        let fields_dict: Bound<'_, PyDict> = schema.get_as_req(intern!(py, "fields"))?;
        let mut fields: Vec<TypedDictField> = Vec::with_capacity(fields_dict.len());

        let cls_name: Option<String> = match schema.get_as_req::<String>(intern!(py, "cls_name")) {
            Ok(name) => Some(name),
            Err(_) => match schema.get_as_req::<Bound<'_, PyType>>(intern!(py, "cls")) {
                Ok(class) => Some(class.getattr(intern!(py, "__name__"))?.extract()?),
                Err(_) => None,
            },
        };

        for (key, value) in fields_dict {
            let field_info = value.downcast::<PyDict>()?;
            let field_name_py = key.downcast_into::<PyString>()?;
            let field_name = field_name_py.to_str()?;

            let schema = field_info.get_as_req(intern!(py, "schema"))?;

            let validator = match build_validator(&schema, config, definitions) {
                Ok(v) => v,
                Err(err) => return py_schema_err!("Field \"{}\":\n  {}", field_name, err),
            };

            let required = match field_info.get_as::<bool>(intern!(py, "required"))? {
                Some(required) => {
                    if required {
                        if let CombinedValidator::WithDefault(ref val) = validator {
                            if val.has_default() {
                                return py_schema_err!(
                                    "Field '{}': a required field cannot have a default value",
                                    field_name
                                );
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
                        return py_schema_err!(
                            "Field '{}': 'on_error = omit' cannot be set for required fields",
                            field_name
                        );
                    }
                }
            }

            let validation_alias = field_info.get_item(intern!(py, "validation_alias"))?;
            let lookup_key_collection = LookupKeyCollection::new(py, validation_alias, field_name)?;

            fields.push(TypedDictField {
                name: field_name.to_string(),
                lookup_key_collection,
                name_py: field_name_py.into(),
                validator,
                required,
            });
        }
        Ok(Self {
            fields,
            extra_behavior,
            extras_validator,
            strict,
            loc_by_alias: config.get_as(intern!(py, "loc_by_alias"))?.unwrap_or(true),
            validate_by_alias: config.get_as(intern!(py, "validate_by_alias"))?,
            validate_by_name: config.get_as(intern!(py, "validate_by_name"))?,
            cls_name,
        }
        .into())
    }
}

impl_py_gc_traverse!(TypedDictValidator {
    fields,
    extras_validator
});

impl Validator for TypedDictValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        let strict = state.strict_or(self.strict);
        let dict = input.validate_dict(strict)?;

        let output_dict = PyDict::new(py);
        let mut errors: Vec<ValLineError> = Vec::with_capacity(self.fields.len());

        let partial_last_key = if state.allow_partial.is_active() {
            dict.last_key().map(Into::into)
        } else {
            None
        };
        let allow_partial = state.allow_partial;

        let validate_by_alias = state.validate_by_alias_or(self.validate_by_alias);
        let validate_by_name = state.validate_by_name_or(self.validate_by_name);

        // we only care about which keys have been used if we're iterating over the object for extra after
        // the first pass
        let mut used_keys: Option<AHashSet<&str>> =
            if self.extra_behavior == ExtraBehavior::Ignore || dict.is_py_get_attr() {
                None
            } else {
                Some(AHashSet::with_capacity(self.fields.len()))
            };

        {
            let state = &mut state.rebind_extra(|extra| extra.data = Some(output_dict.clone()));

            let mut fields_set_count: usize = 0;

            for field in &self.fields {
                let lookup_key = field
                    .lookup_key_collection
                    .select(validate_by_alias, validate_by_name)?;
                let op_key_value = match dict.get_item(lookup_key) {
                    Ok(v) => v,
                    Err(ValError::LineErrors(line_errors)) => {
                        let field_loc: LocItem = field.name.clone().into();
                        if partial_last_key.as_ref() == Some(&field_loc) {
                            for err in line_errors {
                                errors.push(err.with_outer_location(field_loc.clone()));
                            }
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
                    let is_last_partial = if let Some(ref last_key) = partial_last_key {
                        let first_key_loc: LocItem = lookup_path.first_key().into();
                        &first_key_loc == last_key
                    } else {
                        false
                    };
                    state.allow_partial = match is_last_partial {
                        true => allow_partial,
                        false => false.into(),
                    };
                    let state =
                        &mut state.rebind_extra(|extra| extra.field_name = Some(field.name_py.bind(py).clone()));

                    match field.validator.validate(py, value.borrow_input(), state) {
                        Ok(value) => {
                            output_dict.set_item(&field.name_py, value)?;
                            fields_set_count += 1;
                        }
                        Err(ValError::Omit) => continue,
                        Err(ValError::LineErrors(line_errors)) => {
                            if !is_last_partial || field.required {
                                for err in line_errors {
                                    errors.push(lookup_path.apply_error_loc(err, self.loc_by_alias, &field.name));
                                }
                            }
                        }
                        Err(err) => return Err(err),
                    }
                    continue;
                }

                match field.validator.default_value(py, Some(field.name.as_str()), state) {
                    Ok(Some(value)) => {
                        // Default value exists, and passed validation if required
                        output_dict.set_item(&field.name_py, value)?;
                    }
                    Ok(None) => {
                        // This means there was no default value
                        if field.required {
                            errors.push(lookup_key.error(
                                ErrorTypeDefaults::Missing,
                                input,
                                self.loc_by_alias,
                                &field.name,
                            ));
                        }
                    }
                    Err(ValError::Omit) => {}
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

            state.add_fields_set(fields_set_count);
        }

        if let Some(used_keys) = used_keys {
            struct ValidateExtras<'a, 's, 'py> {
                py: Python<'py>,
                used_keys: AHashSet<&'a str>,
                errors: &'a mut Vec<ValLineError>,
                extras_validator: Option<&'a CombinedValidator>,
                output_dict: &'a Bound<'py, PyDict>,
                state: &'a mut ValidationState<'s, 'py>,
                extra_behavior: ExtraBehavior,
                partial_last_key: Option<LocItem>,
                allow_partial: PartialMode,
            }

            impl<'py, Key, Value> ConsumeIterator<ValResult<(Key, Value)>> for ValidateExtras<'_, '_, 'py>
            where
                Key: BorrowInput<'py> + Clone + Into<LocItem>,
                Value: BorrowInput<'py>,
            {
                type Output = ValResult<()>;
                fn consume_iterator(self, iterator: impl Iterator<Item = ValResult<(Key, Value)>>) -> ValResult<()> {
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
                                    let last_partial = self.partial_last_key.as_ref() == Some(&raw_key.clone().into());
                                    self.state.allow_partial = match last_partial {
                                        true => self.allow_partial,
                                        false => false.into(),
                                    };
                                    match validator.validate(self.py, value, self.state) {
                                        Ok(value) => {
                                            self.output_dict.set_item(py_key, value)?;
                                        }
                                        Err(ValError::LineErrors(line_errors)) => {
                                            if !last_partial {
                                                for err in line_errors {
                                                    self.errors.push(err.with_outer_location(raw_key.clone()));
                                                }
                                            }
                                        }
                                        Err(err) => return Err(err),
                                    }
                                } else {
                                    self.output_dict.set_item(py_key, value.to_object(self.py)?)?;
                                }
                            }
                        }
                    }

                    Ok(())
                }
            }

            dict.iterate(ValidateExtras {
                used_keys,
                py,
                errors: &mut errors,
                extras_validator: self.extras_validator.as_deref(),
                output_dict: &output_dict,
                state,
                extra_behavior: self.extra_behavior,
                partial_last_key,
                allow_partial,
            })??;
        }

        if errors.is_empty() {
            Ok(output_dict.into())
        } else {
            Err(ValError::LineErrors(errors))
        }
    }

    fn get_name(&self) -> &str {
        self.cls_name.as_deref().unwrap_or(Self::EXPECTED_TYPE)
    }
}
