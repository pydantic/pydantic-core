use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};
use std::hash::BuildHasherDefault;

use crate::build_tools::{py_error, SchemaDict};
use crate::errors::{ErrorKind, ValError, ValResult};
use crate::input::{GenericArguments, GenericListLike, GenericMapping, Input, JsonInput, JsonObject};
use crate::recursion_guard::{NoHashMap, RecursionGuard};

use super::tuple::TuplePositionalValidator;
use super::typed_dict::TypedDictValidator;
use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct ArgumentsValidator {
    arguments_mapping: Option<NoHashMap<usize, String>>,
    positional_args: Option<TuplePositionalValidator>,
    keyword_args: Option<TypedDictValidator>,
    name: String,
}

impl BuildValidator for ArgumentsValidator {
    const EXPECTED_TYPE: &'static str = "arguments";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let py = schema.py();

        let arguments_mapping = match schema.get_as::<&PyDict>(intern!(py, "arguments_mapping"))? {
            Some(d) => {
                let mut arguments_mapping = NoHashMap::with_hasher(BuildHasherDefault::default());
                for (key, value) in d.iter() {
                    arguments_mapping.insert(key.extract()?, value.extract()?);
                }
                Some(arguments_mapping)
            }
            None => None,
        };

        macro_rules! build_specific_validator {
            ($key:literal, $enum_key:ident) => {
                match schema.get_item(intern!(py, $key)) {
                    Some(sub_schema) => match build_validator(sub_schema, config, build_context)?.0 {
                        CombinedValidator::$enum_key(v) => Some(v),
                        _ => return py_error!("Wrong validator type from {}", $key),
                    },
                    None => None,
                }
            };
        }
        let positional_args = build_specific_validator!("positional_args", TuplePositional);
        let p_args_name = match positional_args {
            Some(ref v) => v.get_name(),
            None => "-",
        };
        let keyword_args = build_specific_validator!("keyword_args", TypedDict);
        let k_args_name = match keyword_args {
            Some(ref v) => v.get_name(),
            None => "-",
        };
        if positional_args.is_none() && keyword_args.is_none() {
            return py_error!("Arguments schema must have either 'positional_args' or 'keyword_args' defined");
        }
        let name = format!("{}[{}, {}]", Self::EXPECTED_TYPE, p_args_name, k_args_name);

        Ok(Self {
            arguments_mapping,
            positional_args,
            keyword_args,
            name,
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
        let mut args = input.validate_args()?;
        self.prepare_args(py, &mut args)?;

        let (pargs, kwargs): (Option<GenericListLike>, Option<GenericMapping>) = match args {
            GenericArguments::Py(op_pargs, op_kwargs) => (
                op_pargs.map(|pargs| pargs.into()),
                op_kwargs.map(|kwargs| kwargs.into()),
            ),
            GenericArguments::Json(op_pargs, op_kwargs) => (
                op_pargs.map(|pargs| pargs.into()),
                op_kwargs.map(|kwargs| kwargs.into()),
            ),
        };

        let arg_result = match (pargs, &self.positional_args) {
            (Some(args), Some(args_validator)) => {
                Some(args_validator.validate_list_like(py, args, input, extra, slots, recursion_guard))
            }
            (Some(_), None) => Some(Err(ValError::new(ErrorKind::UnexpectedPositionalArguments, input))),
            (None, Some(_)) => Some(Err(ValError::new(ErrorKind::MissingPositionalArguments, input))),
            (None, None) => None,
        };

        let kwarg_result = match (kwargs, &self.keyword_args) {
            (Some(args), Some(kwargs_validator)) => {
                Some(kwargs_validator.validate_generic_mapping(py, args, input, extra, slots, recursion_guard))
            }
            (Some(_), None) => Some(Err(ValError::new(ErrorKind::UnexpectedKeywordArguments, input))),
            (None, Some(_)) => Some(Err(ValError::new(ErrorKind::MissingKeywordArguments, input))),
            (None, None) => None,
        };

        match (arg_result, kwarg_result) {
            (Some(Ok(args)), Some(Ok(kwargs))) => Ok((args, kwargs).to_object(py)),
            (Some(Ok(args)), None) => Ok((args, PyDict::new(py)).to_object(py)),
            (None, Some(Ok(kwargs))) => Ok((PyTuple::empty(py), kwargs).to_object(py)),
            (Some(Err(ValError::InternalErr(err))), _) => Err(ValError::InternalErr(err)),
            (_, Some(Err(ValError::InternalErr(err)))) => Err(ValError::InternalErr(err)),
            (Some(Err(ValError::LineErrors(mut args_errors))), Some(Err(ValError::LineErrors(kwargs_errors)))) => {
                args_errors.extend(kwargs_errors);
                Err(ValError::LineErrors(args_errors))
            }
            (Some(Err(args_error)), _) => Err(args_error),
            (_, Some(Err(kwargs_error))) => Err(kwargs_error),
            (None, None) => Ok((PyTuple::empty(py), PyDict::new(py)).to_object(py)),
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

impl ArgumentsValidator {
    /// Move positional arguments to keyword arguments based on mapping.
    fn prepare_args<'s, 'data>(&'s self, py: Python<'data>, arguments: &mut GenericArguments<'data>) -> PyResult<()> {
        if let Some(ref arguments_mapping) = self.arguments_mapping {
            match arguments {
                GenericArguments::Py(Some(pargs), op_kwargs) => {
                    let mut new_args: Vec<&PyAny> = vec![];
                    let kwargs = match op_kwargs {
                        Some(kwargs) => kwargs,
                        None => PyDict::new(py),
                    };
                    for (index, value) in pargs.iter().enumerate() {
                        match arguments_mapping.get(&index) {
                            Some(key) => kwargs.set_item(key, value)?,
                            None => new_args.push(value),
                        }
                    }
                    *arguments = match (new_args.is_empty(), kwargs.is_empty()) {
                        (true, true) => GenericArguments::Py(None, None),
                        (true, false) => GenericArguments::Py(None, Some(kwargs)),
                        (false, true) => GenericArguments::Py(Some(PyList::new(py, new_args)), None),
                        (false, false) => GenericArguments::Py(Some(PyList::new(py, new_args)), Some(kwargs)),
                    }
                }
                _ => (),
            }
        }
        Ok(())
    }
}
