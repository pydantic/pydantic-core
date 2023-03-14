use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString, PyTuple, PyType};

use ahash::AHashSet;

use crate::build_tools::{is_strict, py_err, schema_or_config_same, SchemaDict};
use crate::errors::{ErrorType, ValError, ValLineError, ValResult};
use crate::input::{GenericArguments, Input};
use crate::lookup_key::LookupKey;
use crate::recursion_guard::RecursionGuard;
use crate::validators::function::convert_err;

use super::arguments::{json_get, json_slice, py_get, py_slice};
use super::model::create_class;
use super::with_default::get_default;
use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
struct Field {
    kw_only: bool,
    name: String,
    py_name: Py<PyString>,
    init_only: bool,
    lookup_key: LookupKey,
    validator: CombinedValidator,
}

#[derive(Debug, Clone)]
pub struct DataclassArgsValidator {
    fields: Vec<Field>,
    positional_count: usize,
    init_only_count: Option<usize>,
}

impl BuildValidator for DataclassArgsValidator {
    const EXPECTED_TYPE: &'static str = "dataclass-args";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();

        let populate_by_name = schema_or_config_same(schema, config, intern!(py, "populate_by_name"))?.unwrap_or(false);

        let fields_schema: &PyList = schema.get_as_req(intern!(py, "fields"))?;
        let mut fields: Vec<Field> = Vec::with_capacity(fields_schema.len());

        let mut positional_count = 0;

        for field in fields_schema {
            let field: &PyDict = field.downcast()?;

            let py_name: &PyString = field.get_as_req(intern!(py, "name"))?;
            let name: String = py_name.extract()?;

            let lookup_key = match field.get_item(intern!(py, "validation_alias")) {
                Some(alias) => {
                    let alt_alias = if populate_by_name { Some(name.as_str()) } else { None };
                    LookupKey::from_py(py, alias, alt_alias)?
                }
                None => LookupKey::from_string(py, &name),
            };

            let schema: &PyAny = field.get_as_req(intern!(py, "schema"))?;

            let validator = match build_validator(schema, config, build_context) {
                Ok(v) => v,
                Err(err) => return py_err!("Field '{}':\n  {}", name, err),
            };

            if let CombinedValidator::WithDefault(ref v) = validator {
                if v.omit_on_error() {
                    return py_err!("Field `{}`: omit_on_error cannot be used with arguments", name);
                }
            }

            let kw_only = field.get_as(intern!(py, "kw_only"))?.unwrap_or(true);
            if !kw_only {
                positional_count += 1;
            }

            fields.push(Field {
                kw_only,
                name,
                py_name: py_name.into(),
                lookup_key,
                validator,
                init_only: field.get_as(intern!(py, "init_only"))?.unwrap_or(false),
            });
        }

        let init_only_count = if schema.get_as(intern!(py, "collect_init_only"))?.unwrap_or(false) {
            Some(fields.iter().filter(|f| f.init_only).count())
        } else {
            None
        };

        Ok(Self {
            fields,
            positional_count,
            init_only_count,
        }
        .into())
    }
}

impl Validator for DataclassArgsValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let args = input.validate_dc_args()?;

        let output_dict = PyDict::new(py);
        let mut init_only_args = self.init_only_count.map(Vec::with_capacity);

        let mut errors: Vec<ValLineError> = Vec::new();
        let mut used_keys: AHashSet<&str> = AHashSet::with_capacity(self.fields.len());

        macro_rules! set_item {
            ($field:ident, $value:expr) => {{
                let py_name = $field.py_name.as_ref(py);
                if $field.init_only {
                    if let Some(ref mut init_only_args) = init_only_args {
                        init_only_args.push($value);
                    }
                } else {
                    output_dict.set_item(py_name, $value)?;
                }
            }};
        }

        macro_rules! process {
            ($args:ident, $get_method:ident, $get_macro:ident, $slice_macro:ident) => {{
                // go through fields getting the value from args or kwargs and validating it
                for (index, field) in self.fields.iter().enumerate() {
                    let mut pos_value = None;
                    if let Some(args) = $args.args {
                        if !field.kw_only {
                            pos_value = $get_macro!(args, index);
                        }
                    }

                    let mut kw_value = None;
                    if let Some(kwargs) = $args.kwargs {
                        if let Some((key, value)) = field.lookup_key.$get_method(kwargs)? {
                            used_keys.insert(key);
                            kw_value = Some(value);
                        }
                    }

                    match (pos_value, kw_value) {
                        // found both positional and keyword arguments, error
                        (Some(_), Some(kw_value)) => {
                            errors.push(ValLineError::new_with_loc(
                                ErrorType::MultipleArgumentValues,
                                kw_value,
                                field.name.clone(),
                            ));
                        }
                        // found a positional argument, validate it
                        (Some(pos_value), None) => {
                            match field
                                .validator
                                .validate(py, pos_value, extra, slots, recursion_guard)
                            {
                                Ok(value) => set_item!(field, value),
                                Err(ValError::LineErrors(line_errors)) => {
                                    errors.extend(
                                        line_errors
                                            .into_iter()
                                            .map(|err| err.with_outer_location(index.into())),
                                    );
                                }
                                Err(err) => return Err(err),
                            }
                        }
                        // found a keyword argument, validate it
                        (None, Some(kw_value)) => {
                            match field
                                .validator
                                .validate(py, kw_value, extra, slots, recursion_guard)
                            {
                                Ok(value) => set_item!(field, value),
                                Err(ValError::LineErrors(line_errors)) => {
                                    errors.extend(
                                        line_errors
                                            .into_iter()
                                            .map(|err| err.with_outer_location(field.name.clone().into())),
                                    );
                                }
                                Err(err) => return Err(err),
                            }
                        }
                        // found neither, check if there is a default value, otherwise error
                        (None, None) => {
                            if let Some(value) = get_default(py, &field.validator)? {
                                set_item!(field, value.as_ref().clone_ref(py));
                            } else {
                                errors.push(ValLineError::new_with_loc(
                                    ErrorType::MissingKeywordArgument,
                                    input,
                                    field.name.clone(),
                                ));
                            }
                        }
                    }
                }
                // if there are more args than positional_count, add an error for each one
                if let Some(args) = $args.args {
                    let len = args.len();
                    if len > self.positional_count {
                        for (index, item) in $slice_macro!(args, self.positional_count, len).iter().enumerate() {
                            errors.push(ValLineError::new_with_loc(
                                ErrorType::UnexpectedPositionalArgument,
                                item,
                                index + self.positional_count,
                            ));
                        }
                    }
                }
                // if there are kwargs check any that haven't been processed yet
                if let Some(kwargs) = $args.kwargs {
                    if kwargs.len() != used_keys.len() {
                        for (raw_key, value) in kwargs.iter() {
                            match raw_key.strict_str() {
                                Ok(either_str) => {
                                    if !used_keys.contains(either_str.as_cow()?.as_ref()) {
                                        errors.push(ValLineError::new_with_loc(
                                            ErrorType::UnexpectedKeywordArgument,
                                            value,
                                            raw_key.as_loc_item(),
                                        ));
                                    }
                                }
                                Err(ValError::LineErrors(line_errors)) => {
                                    for err in line_errors {
                                        errors.push(
                                            err.with_outer_location(raw_key.as_loc_item())
                                                .with_type(ErrorType::InvalidKey),
                                        );
                                    }
                                }
                                Err(err) => return Err(err),
                            }
                        }
                    }
                }
            }};
        }
        match args {
            GenericArguments::Py(a) => process!(a, py_get_dict_item, py_get, py_slice),
            GenericArguments::Json(a) => process!(a, json_get, json_get, json_slice),
        }
        if errors.is_empty() {
            if let Some(init_only_args) = init_only_args {
                Ok((output_dict, PyTuple::new(py, init_only_args)).to_object(py))
            } else {
                Ok((output_dict, py.None()).to_object(py))
            }
        } else {
            Err(ValError::LineErrors(errors))
        }
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}

#[derive(Debug, Clone)]
pub struct DataclassValidator {
    strict: bool,
    validator: Box<CombinedValidator>,
    class: Py<PyType>,
    post_init: Option<Py<PyString>>,
    name: String,
}

impl BuildValidator for DataclassValidator {
    const EXPECTED_TYPE: &'static str = "dataclass";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();

        let class: &PyType = schema.get_as_req(intern!(py, "cls"))?;
        let sub_schema: &PyAny = schema.get_as_req(intern!(py, "schema"))?;
        let validator = build_validator(sub_schema, config, build_context)?;

        let post_init = if schema.get_as::<bool>(intern!(py, "post_init"))?.unwrap_or(false) {
            Some(PyString::intern(py, "__post_init__").into_py(py))
        } else {
            None
        };

        Ok(Self {
            strict: is_strict(schema, config)?,
            validator: Box::new(validator),
            class: class.into(),
            post_init,
            // as with model, get the class's `__name__`, not using `class.name()` since it uses `__qualname__`
            // which is not what we want here
            name: class.getattr(intern!(py, "__name__"))?.extract()?,
        }
        .into())
    }
}

impl Validator for DataclassValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let class = self.class.as_ref(py);
        if input.is_exact_instance(class)? {
            Ok(input.to_object(py))
        } else if extra.strict.unwrap_or(self.strict) {
            Err(ValError::new(
                ErrorType::ModelClassType {
                    class_name: self.get_name().to_string(),
                },
                input,
            ))
        } else {
            let output = self.validator.validate(py, input, extra, slots, recursion_guard)?;
            let (dc_dict, post_init_kwargs): (&PyAny, &PyAny) = output.extract(py)?;
            let dc = create_class(self.class.as_ref(py), dc_dict, None)?;

            if let Some(ref post_init) = self.post_init {
                let post_init = post_init.as_ref(py);
                let r = if post_init_kwargs.is_none() {
                    dc.call_method0(py, post_init)
                } else {
                    let args = post_init_kwargs.downcast::<PyTuple>()?;
                    dc.call_method1(py, post_init, args)
                };
                r.map_err(|e| convert_err(py, e, input))?;
            }
            Ok(dc)
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}
