use std::str::FromStr;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString, PyTuple};

use ahash::AHashSet;
use pyo3::IntoPyObjectExt;

use crate::build_tools::py_schema_err;
use crate::build_tools::{schema_or_config_same, ExtraBehavior};
use crate::errors::{ErrorTypeDefaults, ValError, ValLineError, ValResult};
use crate::input::{Arguments, BorrowInput, Input, KeywordArgs, PositionalArgs, ValidatedDict, ValidationMatch};
use crate::lookup_key::LookupKey;
use crate::tools::SchemaDict;

use super::validation_state::ValidationState;
use super::{build_validator, BuildValidator, CombinedValidator, DefinitionsBuilder, Validator};

#[derive(Debug, PartialEq)]
enum ParameterMode {
    PositionalOnly,
    PositionalOrKeyword,
    VarArgs,
    KeywordOnly,
    VarKwargsUniform,
    VarKwargsUnpackedTypedDict,
}

impl FromStr for ParameterMode {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "positional_only" => Ok(Self::PositionalOnly),
            "positional_or_keyword" => Ok(Self::PositionalOrKeyword),
            "var_args" => Ok(Self::VarArgs),
            "keyword_only" => Ok(Self::KeywordOnly),
            "var_kwargs_uniform" => Ok(Self::VarKwargsUniform),
            "var_kwargs_unpacked_typed_dict" => Ok(Self::VarKwargsUnpackedTypedDict),
            s => py_schema_err!("Invalid var_kwargs mode: `{}`", s),
        }
    }
}

#[derive(Debug)]
struct Parameter {
    name: String,
    mode: ParameterMode,
    lookup_key: LookupKey,
    validator: CombinedValidator,
}

impl Parameter {
    fn is_variadic(&self) -> bool {
        match self.mode {
            ParameterMode::VarArgs | ParameterMode::VarKwargsUniform | ParameterMode::VarKwargsUnpackedTypedDict => {
                true
            }
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct ArgumentsV3Validator {
    parameters: Vec<Parameter>,
    positional_params_count: usize,
    loc_by_alias: bool,
    extra: ExtraBehavior,
}

impl BuildValidator for ArgumentsV3Validator {
    const EXPECTED_TYPE: &'static str = "arguments-v2";

    fn build(
        schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();

        let populate_by_name = schema_or_config_same(schema, config, intern!(py, "populate_by_name"))?.unwrap_or(false);

        let arguments_schema: Bound<'_, PyList> = schema.get_as_req(intern!(py, "arguments_schema"))?;
        let mut parameters: Vec<Parameter> = Vec::with_capacity(arguments_schema.len());

        let mut had_default_arg = false;
        let mut had_keyword_only = false;

        for arg in arguments_schema.iter() {
            let arg = arg.downcast::<PyDict>()?;

            let py_name: Bound<PyString> = arg.get_as_req(intern!(py, "name"))?;
            let name = py_name.to_string();
            let py_mode = arg.get_as::<Bound<'_, PyString>>(intern!(py, "mode"))?;
            let py_mode = py_mode
                .as_ref()
                .map(|py_str| py_str.to_str())
                .transpose()?
                .unwrap_or("positional_or_keyword");

            let mode = ParameterMode::from_str(py_mode)?;

            // let positional = mode == "positional_only" || mode == "positional_or_keyword";
            // if positional {
            //     positional_params_count = arg_index + 1;
            // }

            if mode == ParameterMode::KeywordOnly {
                had_keyword_only = true;
            }

            let lookup_key = match arg.get_item(intern!(py, "alias"))? {
                Some(alias) => {
                    let alt_alias = if populate_by_name { Some(name.as_str()) } else { None };
                    LookupKey::from_py(py, &alias, alt_alias)?
                }
                None => LookupKey::from_string(py, &name),
            };

            let schema = arg.get_as_req(intern!(py, "schema"))?;

            let validator = match build_validator(&schema, config, definitions) {
                Ok(v) => v,
                Err(err) => return py_schema_err!("Parameter '{}':\n  {}", name, err),
            };

            let has_default = match validator {
                CombinedValidator::WithDefault(ref v) => {
                    if v.omit_on_error() {
                        return py_schema_err!("Parameter '{}': omit_on_error cannot be used with arguments", name);
                    }
                    v.has_default()
                }
                _ => false,
            };

            if had_default_arg && !has_default && !had_keyword_only {
                return py_schema_err!("Non-default argument '{}' follows default argument", name);
            } else if has_default {
                had_default_arg = true;
            }
            parameters.push(Parameter {
                name,
                mode,
                lookup_key,
                validator,
            });
        }

        let positional_params_count = parameters
            .iter()
            .filter(|p| {
                matches!(
                    p.mode,
                    ParameterMode::PositionalOnly | ParameterMode::PositionalOrKeyword
                )
            })
            .count();

        Ok(Self {
            parameters,
            positional_params_count,
            loc_by_alias: config.get_as(intern!(py, "loc_by_alias"))?.unwrap_or(true),
            extra: ExtraBehavior::from_schema_or_config(py, schema, config, ExtraBehavior::Forbid)?,
        }
        .into())
    }
}

impl_py_gc_traverse!(Parameter { validator });

impl_py_gc_traverse!(ArgumentsV3Validator { parameters });

impl ArgumentsV3Validator {
    /// Validate the arguments from a mapping:
    /// ```py
    /// def func(a: int, /, *, b: str, **kwargs: int) -> None:
    ///     ...
    ///
    /// valid_mapping = {'a': 1, 'b': 'test', 'kwargs': {'c': 1, 'd': 2}}
    /// ```
    fn validate_from_mapping<'py>(
        &self,
        py: Python<'py>,
        original_input: &(impl Input<'py> + ?Sized),
        mapping: impl ValidatedDict<'py>,
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        let mut output_args: Vec<PyObject> = Vec::with_capacity(self.positional_params_count);
        let output_kwargs = PyDict::new(py);
        let mut errors: Vec<ValLineError> = Vec::new();

        for parameter in self.parameters.iter() {
            // A value is present in the mapping:
            if let Some((lookup_path, dict_value)) = mapping.get_item(&parameter.lookup_key)? {
                match parameter.mode {
                    ParameterMode::PositionalOnly | ParameterMode::PositionalOrKeyword => {
                        match parameter.validator.validate(py, dict_value.borrow_input(), state) {
                            Ok(value) => output_args.push(value),
                            Err(ValError::LineErrors(line_errors)) => {
                                errors.extend(
                                    line_errors.into_iter().map(|err| {
                                        lookup_path.apply_error_loc(err, self.loc_by_alias, &parameter.name)
                                    }),
                                );
                            }
                            Err(err) => return Err(err),
                        }
                    }
                    // ParameterMode::VarArgs => match dict_value.validate_tuple() {
                    //     Ok(iterable) => for value in iterable,
                    //     Err(err) => return Err(err),
                    // },
                    ParameterMode::VarArgs => todo!(),
                    ParameterMode::KeywordOnly => {
                        match parameter.validator.validate(py, dict_value.borrow_input(), state) {
                            Ok(value) => {
                                output_kwargs.set_item(PyString::new(py, parameter.name.as_str()).unbind(), value)?
                            }
                            Err(ValError::LineErrors(line_errors)) => {
                                errors.extend(
                                    line_errors.into_iter().map(|err| {
                                        lookup_path.apply_error_loc(err, self.loc_by_alias, &parameter.name)
                                    }),
                                );
                            }
                            Err(err) => return Err(err),
                        }
                    }
                    ParameterMode::VarKwargsUniform => match dict_value.borrow_input().as_kwargs(py) {
                        // We will validate that keys are strings, and values match the validator:
                        Some(value) => {
                            for (dict_key, dict_value) in value.into_iter() {
                                // Validate keys are strings:
                                match dict_key.validate_str(true, false).map(ValidationMatch::into_inner) {
                                    Ok(_) => (),
                                    Err(ValError::LineErrors(line_errors)) => {
                                        for err in line_errors {
                                            errors.push(
                                                err.with_outer_location(dict_key.clone())
                                                    .with_type(ErrorTypeDefaults::InvalidKey),
                                            );
                                        }
                                        continue;
                                    }
                                    Err(err) => return Err(err),
                                }
                                // Validate values:
                                match parameter.validator.validate(py, dict_value.borrow_input(), state) {
                                    Ok(value) => output_kwargs.set_item(dict_key, value)?,
                                    Err(ValError::LineErrors(line_errors)) => {
                                        errors.extend(line_errors.into_iter().map(|err| {
                                            lookup_path.apply_error_loc(err, self.loc_by_alias, &parameter.name)
                                        }));
                                    }
                                    Err(err) => return Err(err),
                                }
                            }
                        }
                        None => todo!(),
                    },
                    ParameterMode::VarKwargsUnpackedTypedDict => {
                        let kwargs_dict = dict_value
                            .borrow_input()
                            .as_kwargs(py)
                            .unwrap_or_else(|| PyDict::new(py));
                        match parameter.validator.validate(py, kwargs_dict.as_any(), state) {
                            Ok(value) => {
                                output_kwargs.update(value.downcast_bound::<PyDict>(py).unwrap().as_mapping())?;
                            }
                            Err(ValError::LineErrors(line_errors)) => {
                                errors.extend(line_errors);
                            }
                            Err(err) => return Err(err),
                        }
                    }
                }
            // No value is present in the mapping, fallback to the default value (and error if no default):
            } else {
                match parameter.mode {
                    ParameterMode::PositionalOnly | ParameterMode::PositionalOrKeyword | ParameterMode::KeywordOnly => {
                        if let Some(value) =
                            parameter
                                .validator
                                .default_value(py, Some(parameter.name.as_str()), state)?
                        {
                            if parameter.mode == ParameterMode::PositionalOnly {
                                output_args.push(value);
                            } else {
                                output_kwargs.set_item(PyString::new(py, parameter.name.as_str()).unbind(), value)?;
                            }
                        } else {
                            let error_type = match parameter.mode {
                                ParameterMode::PositionalOnly => ErrorTypeDefaults::MissingPositionalOnlyArgument,
                                ParameterMode::PositionalOrKeyword => ErrorTypeDefaults::MissingArgument,
                                ParameterMode::KeywordOnly => ErrorTypeDefaults::MissingKeywordOnlyArgument,
                                _ => unreachable!(),
                            };

                            errors.push(parameter.lookup_key.error(
                                error_type,
                                original_input,
                                self.loc_by_alias,
                                &parameter.name,
                            ));
                        }
                    }
                    // Variadic args/kwargs can be empty by definition:
                    _ => (),
                }
            }
        }

        if !errors.is_empty() {
            return Err(ValError::LineErrors(errors));
        } else {
            return Ok((PyTuple::new(py, output_args)?, output_kwargs).into_py_any(py)?);
        }
    }

    /// Validate the arguments from an [`ArgsKwargs`][crate::argument_markers::ArgsKwargs] instance:
    /// ```py
    /// def func(a: int, /, *, b: str, **kwargs: int) -> None:
    ///     ...
    ///
    /// valid_argskwargs = ArgsKwargs((1,), {'b': 'test', 'c': 1, 'd': 2})
    /// ```
    fn validate_from_argskwargs<'py>(
        &self,
        py: Python<'py>,
        original_input: &(impl Input<'py> + ?Sized),
        args_kwargs: impl Arguments<'py>,
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        let mut output_args: Vec<PyObject> = Vec::with_capacity(self.positional_params_count);
        let output_kwargs = PyDict::new(py);
        let mut errors: Vec<ValLineError> = Vec::new();
        let mut used_kwargs: AHashSet<&str> = AHashSet::with_capacity(self.parameters.len());

        // go through non variadic arguments, getting the value from args or kwargs and validating it
        for (index, parameter) in self.parameters.iter().filter(|p| !p.is_variadic()).enumerate() {
            let mut pos_value = None;
            if let Some(args) = args_kwargs.args() {
                if matches!(
                    parameter.mode,
                    ParameterMode::PositionalOnly | ParameterMode::PositionalOrKeyword
                ) {
                    pos_value = args.get_item(index);
                }
            }

            let mut kw_value = None;
            if let Some(kwargs) = args_kwargs.kwargs() {
                if matches!(
                    parameter.mode,
                    ParameterMode::PositionalOrKeyword | ParameterMode::KeywordOnly
                ) {
                    if let Some((lookup_path, value)) = kwargs.get_item(&parameter.lookup_key)? {
                        used_kwargs.insert(lookup_path.first_key());
                        kw_value = Some((lookup_path, value));
                    }
                }
            }

            match (pos_value, kw_value) {
                (Some(_), Some((_, kw_value))) => {
                    errors.push(ValLineError::new_with_loc(
                        ErrorTypeDefaults::MultipleArgumentValues,
                        kw_value.borrow_input(),
                        parameter.name.clone(),
                    ));
                }
                (Some(pos_value), None) => match parameter.validator.validate(py, pos_value.borrow_input(), state) {
                    Ok(value) => output_args.push(value),
                    Err(ValError::LineErrors(line_errors)) => {
                        errors.extend(line_errors.into_iter().map(|err| err.with_outer_location(index)));
                    }
                    Err(err) => return Err(err),
                },
                (None, Some((lookup_path, kw_value))) => {
                    match parameter.validator.validate(py, kw_value.borrow_input(), state) {
                        Ok(value) => {
                            output_kwargs.set_item(PyString::new(py, parameter.name.as_str()).unbind(), value)?
                        }
                        Err(ValError::LineErrors(line_errors)) => {
                            errors.extend(
                                line_errors
                                    .into_iter()
                                    .map(|err| lookup_path.apply_error_loc(err, self.loc_by_alias, &parameter.name)),
                            );
                        }
                        Err(err) => return Err(err),
                    }
                }
                (None, None) => {
                    if let Some(value) = parameter
                        .validator
                        .default_value(py, Some(parameter.name.as_str()), state)?
                    {
                        if matches!(
                            parameter.mode,
                            ParameterMode::PositionalOnly | ParameterMode::PositionalOrKeyword
                        ) {
                            output_kwargs.set_item(PyString::new(py, parameter.name.as_str()).unbind(), value)?
                        } else {
                            output_args.push(value);
                        }
                    } else {
                        // Required and no default, error:
                        match parameter.mode {
                            ParameterMode::PositionalOnly => {
                                errors.push(ValLineError::new_with_loc(
                                    ErrorTypeDefaults::MissingPositionalOnlyArgument,
                                    original_input,
                                    index,
                                ));
                            }
                            ParameterMode::PositionalOrKeyword => {
                                errors.push(parameter.lookup_key.error(
                                    ErrorTypeDefaults::MissingArgument,
                                    original_input,
                                    self.loc_by_alias,
                                    &parameter.name,
                                ));
                            }
                            ParameterMode::KeywordOnly => {
                                errors.push(parameter.lookup_key.error(
                                    ErrorTypeDefaults::MissingKeywordOnlyArgument,
                                    original_input,
                                    self.loc_by_alias,
                                    &parameter.name,
                                ));
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
        }

        // if there are args check any where index > positional_params_count since they won't have been checked yet
        if let Some(args) = args_kwargs.args() {
            let len = args.len();
            if len > self.positional_params_count {
                if let Some(var_args_param) = self.parameters.iter().find(|p| p.mode == ParameterMode::VarArgs) {
                    for (index, item) in args.iter().enumerate().skip(self.positional_params_count) {
                        match var_args_param.validator.validate(py, item.borrow_input(), state) {
                            Ok(value) => output_args.push(value),
                            Err(ValError::LineErrors(line_errors)) => {
                                errors.extend(line_errors.into_iter().map(|err| err.with_outer_location(index)));
                            }
                            Err(err) => return Err(err),
                        }
                    }
                } else {
                    for (index, item) in args.iter().enumerate().skip(self.positional_params_count) {
                        errors.push(ValLineError::new_with_loc(
                            ErrorTypeDefaults::UnexpectedPositionalArgument,
                            item,
                            index,
                        ));
                    }
                }
            }
        }

        let remaining_kwargs = PyDict::new(py);

        // if there are kwargs check any that haven't been processed yet
        if let Some(kwargs) = args_kwargs.kwargs() {
            if kwargs.len() > used_kwargs.len() {
                for result in kwargs.iter() {
                    let (raw_key, value) = result?;
                    let either_str = match raw_key
                        .borrow_input()
                        .validate_str(true, false)
                        .map(ValidationMatch::into_inner)
                    {
                        Ok(k) => k,
                        Err(ValError::LineErrors(line_errors)) => {
                            for err in line_errors {
                                errors.push(
                                    err.with_outer_location(raw_key.clone())
                                        .with_type(ErrorTypeDefaults::InvalidKey),
                                );
                            }
                            continue;
                        }
                        Err(err) => return Err(err),
                    };
                    if !used_kwargs.contains(either_str.as_cow()?.as_ref()) {
                        let maybe_var_kwargs_parameter = self.parameters.iter().find(|p| {
                            matches!(
                                p.mode,
                                ParameterMode::VarKwargsUniform | ParameterMode::VarKwargsUnpackedTypedDict
                            )
                        });

                        match maybe_var_kwargs_parameter {
                            None => {
                                if self.extra == ExtraBehavior::Forbid {
                                    errors.push(ValLineError::new_with_loc(
                                        ErrorTypeDefaults::UnexpectedKeywordArgument,
                                        value,
                                        raw_key.clone(),
                                    ));
                                }
                            }
                            Some(var_kwargs_parameter) => {
                                match var_kwargs_parameter.mode {
                                    ParameterMode::VarKwargsUniform => {
                                        match var_kwargs_parameter.validator.validate(py, value.borrow_input(), state) {
                                            Ok(value) => {
                                                output_kwargs
                                                    .set_item(either_str.as_py_string(py, state.cache_str()), value)?;
                                            }
                                            Err(ValError::LineErrors(line_errors)) => {
                                                for err in line_errors {
                                                    errors.push(err.with_outer_location(raw_key.clone()));
                                                }
                                            }
                                            Err(err) => return Err(err),
                                        }
                                    }
                                    ParameterMode::VarKwargsUnpackedTypedDict => {
                                        // Save to the remaining kwargs, we will validate as a single dict:
                                        remaining_kwargs.set_item(
                                            either_str.as_py_string(py, state.cache_str()),
                                            value.borrow_input().to_object(py)?,
                                        )?;
                                    }
                                    _ => unreachable!(),
                                }
                            }
                        }
                    }
                }
            }
        }

        if !remaining_kwargs.is_empty() {
            // In this case, the unpacked typeddict var kwargs parameter is guaranteed to exist:
            let var_kwargs_parameter = self
                .parameters
                .iter()
                .find(|p| p.mode == ParameterMode::VarKwargsUnpackedTypedDict)
                .unwrap();
            match var_kwargs_parameter
                .validator
                .validate(py, remaining_kwargs.as_any(), state)
            {
                Ok(value) => {
                    output_kwargs.update(value.downcast_bound::<PyDict>(py).unwrap().as_mapping())?;
                }
                Err(ValError::LineErrors(line_errors)) => {
                    errors.extend(line_errors);
                }
                Err(err) => return Err(err),
            }
        }

        if !errors.is_empty() {
            return Err(ValError::LineErrors(errors));
        } else {
            return Ok((PyTuple::new(py, output_args)?, output_kwargs).into_py_any(py)?);
        }
    }
}

impl Validator for ArgumentsV3Validator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        // this validator does not yet support partial validation, disable it to avoid incorrect results
        state.allow_partial = false.into();

        let args_dict = input.validate_dict(false);

        // Validation from a dictionary, mapping parameter names to the values:
        if let Ok(dict) = args_dict {
            return self.validate_from_mapping(py, input, dict, state);
        } else {
            let args = input.validate_args_v3()?;
            return self.validate_from_argskwargs(py, input, args, state);
        }
    }

    fn get_name(&self) -> &str {
        Self::EXPECTED_TYPE
    }
}
