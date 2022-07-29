use ahash::AHashSet;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString, PyTuple};

use crate::build_tools::{py_error, schema_or_config_same, SchemaDict};
use crate::errors::{ErrorKind, ValError, ValLineError, ValResult};
use crate::input::{GenericArguments, Input, PyArgs};
use crate::lookup_key::LookupKey;
use crate::recursion_guard::RecursionGuard;
use crate::SchemaError;

use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
struct Argument {
    positional: bool,
    name: String,
    kw_lookup_key: Option<LookupKey>,
    kwarg_key: Option<Py<PyString>>,
    default: Option<PyObject>,
    default_factory: Option<PyObject>,
    validator: CombinedValidator,
}

#[derive(Debug, Clone)]
pub struct ArgumentsValidator {
    arguments: Vec<Argument>,
    positional_args_count: usize,
    var_args_validator: Option<Box<CombinedValidator>>,
    var_kwargs_validator: Option<Box<CombinedValidator>>,
}

impl BuildValidator for ArgumentsValidator {
    const EXPECTED_TYPE: &'static str = "arguments";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();

        let populate_by_name = schema_or_config_same(schema, config, intern!(py, "populate_by_name"))?.unwrap_or(false);

        let arguments_list: &PyList = schema.get_as_req(intern!(py, "arguments"))?;
        let mut arguments: Vec<Argument> = Vec::with_capacity(arguments_list.len());

        let mut positional_args_count = 0;

        for (arg_index, arg) in arguments_list.iter().enumerate() {
            let arg: &PyDict = arg.cast_as()?;

            let name: String = arg.get_as_req(intern!(py, "name"))?;
            let mode: &str = arg.get_as_req(intern!(py, "mode"))?;
            let positional = mode == "positional_only" || mode == "positional_or_keyword";
            if positional {
                positional_args_count = arg_index;
            }

            let mut kw_lookup_key = None;
            let mut kwarg_key = None;
            if mode == "keyword_only" || mode == "positional_or_keyword" {
                kw_lookup_key = match arg.get_item(intern!(py, "alias")) {
                    Some(alias) => {
                        let alt_alias = if populate_by_name { Some(name.as_str()) } else { None };
                        Some(LookupKey::from_py(py, alias, alt_alias)?)
                    }
                    None => Some(LookupKey::from_string(py, &name)),
                };
                kwarg_key = Some(PyString::intern(py, &name).into());
            }

            let schema: &PyAny = arg
                .get_as_req(intern!(py, "schema"))
                .map_err(|err| SchemaError::new_err(format!("Argument \"{}\":\n  {}", name, err)))?;

            let validator = match build_validator(schema, config, build_context) {
                Ok((v, _)) => v,
                Err(err) => return py_error!("Argument \"{}\":\n  {}", name, err),
            };

            let (default, default_factory) = match (
                arg.get_as(intern!(py, "default"))?,
                arg.get_as(intern!(py, "default_factory"))?,
            ) {
                (Some(_default), Some(_default_factory)) => {
                    return py_error!("'default' and 'default_factory' cannot be used together")
                }
                (default, default_factory) => (default, default_factory),
            };
            arguments.push(Argument {
                positional,
                kw_lookup_key,
                name,
                kwarg_key,
                default,
                default_factory,
                validator,
            });
        }

        Ok(Self {
            arguments,
            positional_args_count,
            var_args_validator: match schema.get_item(intern!(py, "var_args_schema")) {
                Some(v) => Some(Box::new(build_validator(v, config, build_context)?.0)),
                None => None,
            },
            var_kwargs_validator: match schema.get_item(intern!(py, "var_kwargs_validator")) {
                Some(v) => Some(Box::new(build_validator(v, config, build_context)?.0)),
                None => None,
            },
        }
        .into())
    }
}

impl Validator for ArgumentsValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let args = input.validate_args()?;
        match args {
            GenericArguments::Py(args) => self.validate_py_args(py, args, input, extra, slots, recursion_guard),
            GenericArguments::Json(_args) => todo!(),
        }
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}

impl ArgumentsValidator {
    fn validate_py_args<'s, 'data>(
        &'s self,
        py: Python<'data>,
        py_args: PyArgs<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let mut output_args: Vec<PyObject> = Vec::with_capacity(self.positional_args_count);
        let output_kwargs = PyDict::new(py);
        let mut errors: Vec<ValLineError> = Vec::new();
        let mut used_args = 0;
        let mut used_kwargs: AHashSet<&str> = AHashSet::with_capacity(self.arguments.len());

        for (index, argument_info) in self.arguments.iter().enumerate() {
            let mut pos_value: Option<&PyAny> = None;
            if let Some(args) = py_args.args {
                if argument_info.positional {
                    pos_value = args.get_item(index).ok();
                    used_args = index;
                }
            }
            let mut kw_value: Option<&PyAny> = None;
            if let Some(kwargs) = py_args.kwargs {
                if let Some(ref lookup_key) = argument_info.kw_lookup_key {
                    if let Some((key, value)) = lookup_key.py_get_item(kwargs)? {
                        used_kwargs.insert(key);
                        kw_value = Some(value);
                    }
                }
            }

            match (pos_value, kw_value) {
                (Some(_), Some(_)) => {
                    errors.push(ValLineError::new_with_loc(
                        ErrorKind::MultipleArgumentValues {
                            arg: argument_info.name.clone(),
                        },
                        input,
                        index,
                    ));
                }
                (Some(pos_value), None) => {
                    match argument_info
                        .validator
                        .validate(py, pos_value, extra, slots, recursion_guard)
                    {
                        Ok(value) => output_args.push(value),
                        Err(ValError::LineErrors(line_errors)) => {
                            errors.extend(line_errors.into_iter().map(|err| err.with_outer_location(index.into())));
                        }
                        Err(err) => return Err(err),
                    }
                }
                (None, Some(kw_value)) => {
                    match argument_info
                        .validator
                        .validate(py, kw_value, extra, slots, recursion_guard)
                    {
                        Ok(value) => output_kwargs.set_item(argument_info.kwarg_key.as_ref().unwrap(), value)?,
                        Err(ValError::LineErrors(line_errors)) => {
                            errors.extend(
                                line_errors
                                    .into_iter()
                                    .map(|err| err.with_outer_location(argument_info.name.clone().into())),
                            );
                        }
                        Err(err) => return Err(err),
                    }
                }
                (None, None) => {
                    if let Some(ref default) = argument_info.default {
                        if let Some(ref kwarg_key) = argument_info.kwarg_key {
                            output_kwargs.set_item(kwarg_key, default)?;
                        } else {
                            output_args.push(default.clone_ref(py));
                        }
                    } else if let Some(ref default_factory) = argument_info.default_factory {
                        let default = default_factory.call0(py)?;
                        if let Some(ref kwarg_key) = argument_info.kwarg_key {
                            output_kwargs.set_item(kwarg_key, default)?;
                        } else {
                            output_args.push(default);
                        }
                    } else if argument_info.kwarg_key.is_some() {
                        errors.push(ValLineError::new_with_loc(
                            ErrorKind::MissingArgument,
                            input,
                            argument_info.name.clone(),
                        ));
                    } else {
                        errors.push(ValLineError::new_with_loc(ErrorKind::MissingArgument, input, index));
                    };
                }
            }
        }
        if let Some(args) = py_args.args {
            let len = args.len();
            // TODO can we use self.positional_args_count instead of used_args? I think so.
            if len > used_args {
                if let Some(ref validator) = self.var_args_validator {
                    for (index, item) in args.get_slice(used_args, len).iter().enumerate() {
                        match validator.validate(py, item, extra, slots, recursion_guard) {
                            Ok(value) => output_args.push(value),
                            Err(ValError::LineErrors(line_errors)) => {
                                errors.extend(
                                    line_errors
                                        .into_iter()
                                        .map(|err| err.with_outer_location((index + used_args).into())),
                                );
                            }
                            Err(err) => return Err(err),
                        }
                    }
                } else {
                    errors.push(ValLineError::new(
                        ErrorKind::UnexpectedPositionalArguments {
                            unexpected_count: len - used_args,
                        },
                        input,
                    ));
                }
            }
        }
        if let Some(kwargs) = py_args.kwargs {
            for (raw_key, value) in kwargs.iter() {
                let either_str = match raw_key.strict_str() {
                    Ok(k) => k,
                    Err(ValError::LineErrors(line_errors)) => {
                        for err in line_errors {
                            errors.push(
                                err.with_outer_location(raw_key.as_loc_item())
                                    .with_kind(ErrorKind::InvalidKey),
                            );
                        }
                        continue;
                    }
                    Err(err) => return Err(err),
                };
                if !used_kwargs.contains(either_str.as_cow().as_ref()) {
                    match self.var_kwargs_validator {
                        Some(ref validator) => match validator.validate(py, value, extra, slots, recursion_guard) {
                            Ok(value) => output_kwargs.set_item(either_str.as_py_string(py), value)?,
                            Err(ValError::LineErrors(line_errors)) => {
                                for err in line_errors {
                                    errors.push(err.with_outer_location(raw_key.as_loc_item()));
                                }
                            }
                            Err(err) => return Err(err),
                        },
                        None => {
                            errors.push(ValLineError::new_with_loc(
                                ErrorKind::UnexpectedKeywordArgument,
                                value,
                                raw_key.as_loc_item(),
                            ));
                        }
                    }
                }
            }
        }
        Ok((PyTuple::new(py, output_args), output_kwargs).to_object(py))
    }
}
