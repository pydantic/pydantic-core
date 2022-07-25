use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

use crate::build_tools::{py_error, SchemaDict};
use crate::errors::{ErrorKind, ValError, ValResult};
use crate::input::{GenericArguments, GenericListLike, GenericMapping, Input};
use crate::recursion_guard::RecursionGuard;

use super::tuple::TuplePositionalValidator;
use super::typed_dict::TypedDictValidator;
use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct ArgumentsValidator {
    arguments_mapping: Option<(usize, Vec<(usize, String)>)>,
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
                let arguments_mapping = d
                    .iter()
                    .map(|(k, v)| {
                        let k = k.extract()?;
                        let v = v.extract()?;
                        Ok((k, v))
                    })
                    .collect::<PyResult<Vec<_>>>()?;
                match arguments_mapping.first() {
                    Some((s, _)) => Some((*s, arguments_mapping)),
                    None => None,
                }
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
        let positional_args = build_specific_validator!("positional_args_schema", TuplePositional);
        let p_args_name = match positional_args {
            Some(ref v) => v.get_name(),
            None => "-",
        };
        let keyword_args = build_specific_validator!("keyword_args_schema", TypedDict);
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
        let args = input.validate_args()?;
        let args = self.prepare_args(py, args)?;

        let (pargs, kwargs): (Option<GenericListLike>, GenericMapping) = match args {
            GenericArguments::Py(op_pargs, kwargs) => (op_pargs.map(|pargs| pargs.into()), kwargs.into()),
            GenericArguments::Json(op_pargs, kwargs) => (op_pargs.map(|pargs| pargs.into()), kwargs.into()),
        };

        let arg_result = match (pargs, &self.positional_args) {
            (Some(args), Some(args_validator)) => {
                Some(args_validator.validate_list_like(py, args, input, extra, slots, recursion_guard))
            }
            (Some(pa), None) => match pa.generic_len() {
                0 => None,
                _ => Some(Err(ValError::new(ErrorKind::UnexpectedPositionalArguments, input))),
            },
            (None, Some(_)) => Some(Err(ValError::new(ErrorKind::MissingPositionalArguments, input))),
            (None, None) => None,
        };

        let kwarg_result = match (kwargs, &self.keyword_args) {
            (kwargs, Some(kwargs_validator)) => {
                Some(kwargs_validator.validate_generic_mapping(py, kwargs, input, extra, slots, recursion_guard))
            }
            (kwargs, None) => match kwargs.generic_len()? {
                0 => None,
                _ => Some(Err(ValError::new(ErrorKind::UnexpectedKeywordArguments, input))),
            },
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
    fn prepare_args<'s, 'data>(
        &'s self,
        py: Python<'data>,
        mut arguments: GenericArguments<'data>,
    ) -> PyResult<GenericArguments<'data>> {
        if let Some((slice_at, ref arguments_mapping)) = self.arguments_mapping {
            match arguments {
                GenericArguments::Py(Some(pargs), kwargs) => {
                    let new_pargs = pargs.get_slice(0, slice_at);
                    // have to copy the kwargs so we don't modify the input dict
                    let kwargs = kwargs.copy()?;

                    for (index, key) in arguments_mapping {
                        if let Ok(value) = pargs.get_item(*index) {
                            kwargs.set_item(key, value)?;
                        } else {
                            break;
                        }
                    }
                    Ok(GenericArguments::Py(Some(new_pargs), kwargs))
                }
                GenericArguments::Json(Some(pargs), mut kwargs) => {
                    let new_pargs = &pargs[..slice_at];

                    for (index, key) in arguments_mapping {
                        if let Some(value) = pargs.get(*index) {
                            kwargs.insert(key.clone(), value.clone());
                        } else {
                            break;
                        }
                    }
                    Ok(GenericArguments::Json(Some(new_pargs), kwargs))
                }
                _ => Ok(arguments),
            }
        } else {
            Ok(arguments)
        }
    }
}
