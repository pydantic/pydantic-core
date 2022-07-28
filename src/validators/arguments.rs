use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};

use crate::build_tools::{py_error, SchemaDict};
use crate::errors::{ErrorKind, ValError, ValLineError, ValResult};
use crate::input::{json_object_to_py, GenericArguments, GenericListLike, GenericMapping, Input};
use crate::recursion_guard::RecursionGuard;

use super::tuple::TuplePositionalValidator;
use super::typed_dict::{IterAttributes, TypedDictValidator};
use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct ArgumentsValidator {
    arguments_mapping: Option<(usize, Vec<(usize, String)>)>,
    pargs_validator: Option<TuplePositionalValidator>,
    kwargs_validator: Option<TypedDictValidator>,
    always_validate_kwargs: bool,
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
                #[allow(clippy::manual_map)]
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
        let pargs_validator = build_specific_validator!("positional_args_schema", TuplePositional);
        let p_args_name = match pargs_validator {
            Some(ref v) => v.get_name(),
            None => "-",
        };
        let kwargs_validator = build_specific_validator!("keyword_args_schema", TypedDict);
        let k_args_name = match kwargs_validator {
            Some(ref v) => v.get_name(),
            None => "-",
        };
        if pargs_validator.is_none() && kwargs_validator.is_none() {
            return py_error!(
                "Arguments schema must have either 'positional_args_schema' or 'keyword_args_schema' defined"
            );
        }
        let name = format!("{}[{}, {}]", Self::EXPECTED_TYPE, p_args_name, k_args_name);
        let always_validate_kwargs = match kwargs_validator {
            Some(ref v) => v.has_optional_fields(),
            None => false,
        };

        Ok(Self {
            arguments_mapping,
            pargs_validator,
            kwargs_validator,
            always_validate_kwargs,
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
        let args = self.build_args(py, input)?;

        let (pargs, kwargs): (Option<GenericListLike>, Option<GenericMapping>) = match args {
            GenericArguments::Py(op_pargs, kwargs) => {
                (op_pargs.map(|pargs| pargs.into()), kwargs.map(|pargs| pargs.into()))
            }
            GenericArguments::Json(op_pargs, kwargs) => {
                (op_pargs.map(|pargs| pargs.into()), kwargs.map(|pargs| pargs.into()))
            }
        };

        let arg_result = match (pargs, &self.pargs_validator) {
            (Some(args), Some(args_validator)) => Some(
                args_validator
                    .validate_list_like(py, args, input, extra, slots, recursion_guard)
                    .map_err(map_pargs_errors),
            ),
            (Some(pa), None) => match pa.generic_len() {
                0 => None,
                unexpected_count => Some(Err(ValError::new(
                    ErrorKind::UnexpectedPositionalArguments { unexpected_count },
                    input,
                ))),
            },
            (None, Some(args_validator)) => match args_validator.len() {
                0 => None,
                args_count => Some(Err(ValError::LineErrors(
                    (0..args_count)
                        .map(|index| ValLineError::new_with_loc(ErrorKind::MissingPositionalArgument, input, index))
                        .collect(),
                ))),
            },
            (None, None) => None,
        };

        let kwarg_result = match (kwargs, &self.kwargs_validator) {
            (Some(kwargs), Some(kwargs_validator)) => Some(
                kwargs_validator
                    .validate_generic_mapping(py, kwargs, input, extra, slots, recursion_guard)
                    .map_err(map_kwargs_errors),
            ),
            (Some(kwargs), None) => match kwargs.generic_len()? {
                0 => None,
                _ => Some(Err(self.unexpected_kwargs(kwargs))),
            },
            (None, Some(kwargs_validator)) => {
                if self.always_validate_kwargs {
                    let kwargs = GenericMapping::PyDict(PyDict::new(py));
                    Some(
                        kwargs_validator
                            .validate_generic_mapping(py, kwargs, input, extra, slots, recursion_guard)
                            .map_err(map_kwargs_errors),
                    )
                } else {
                    Some(Err(ValError::LineErrors(
                        kwargs_validator
                            .keys()
                            .into_iter()
                            .map(|key| ValLineError::new_with_loc(ErrorKind::MissingKeywordArgument, input, key))
                            .collect(),
                    )))
                }
            }
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
    fn build_args<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
    ) -> ValResult<'data, GenericArguments<'data>> {
        let mut args = input.validate_args()?;

        if let Some((slice_at, ref arguments_mapping)) = self.arguments_mapping {
            match args {
                GenericArguments::Py(Some(pargs), kwargs) => {
                    let new_pargs = pargs.get_slice(0, slice_at);
                    let kwargs = match kwargs {
                        // have to copy the kwargs so we don't modify the input dict
                        Some(kw) => kw.copy()?,
                        None => PyDict::new(py),
                    };

                    for (index, key) in arguments_mapping {
                        if let Ok(value) = pargs.get_item(*index) {
                            kwargs.set_item(key, value)?;
                        } else {
                            break;
                        }
                    }
                    args = GenericArguments::Py(Some(new_pargs), Some(kwargs));
                }
                GenericArguments::Json(Some(pargs), kwargs) => {
                    // TODO ideally we wouldn't have to fallback to python objects here, but instead could continue
                    // to operate on JsonInput and JsonArray/JsonObject, but all the approaches I tried failed:
                    // * creating a new JsonObject allowed it to be mutated but `validate_generic_mapping` needs
                    //   a reference and the reference has the wrong lifetime
                    // * if we try to mutate kwargs directly we run into problems as it's a reference, not a
                    //   mutable reference to make it editable we'd have to make input mutable everywhere which seems
                    //   ugly
                    let pargs_slice = &pargs[..slice_at];
                    let py_pargs = match pargs_slice.is_empty() {
                        true => None,
                        false => Some(PyList::new(py, pargs_slice)),
                    };

                    let py_kwargs = match kwargs {
                        // have to copy the kwargs so we don't modify the input dict
                        Some(kw) => json_object_to_py(kw, py),
                        None => PyDict::new(py),
                    };
                    for (index, key) in arguments_mapping {
                        if let Some(value) = pargs.get(*index) {
                            py_kwargs.set_item(key, value.to_object(py))?;
                        } else {
                            break;
                        }
                    }
                    args = GenericArguments::Py(py_pargs, Some(py_kwargs));
                }
                _ => (),
            }
        }
        Ok(args)
    }

    pub fn unexpected_kwargs<'s, 'data>(&'s self, dict: GenericMapping<'data>) -> ValError<'data> {
        macro_rules! collect_errors {
            ($iter:expr) => {{
                $iter
                    .map(|(k, v)| ValLineError::new_with_loc(ErrorKind::UnexpectedKeywordArgument, v, k.as_loc_item()))
                    .collect()
            }};
        }
        let errors: Vec<ValLineError> = match dict {
            GenericMapping::PyDict(d) => collect_errors!(d.iter()),
            GenericMapping::PyGetAttr(d) => collect_errors!(IterAttributes::new(d)),
            GenericMapping::JsonObject(d) => collect_errors!(d.iter()),
        };
        ValError::LineErrors(errors)
    }
}

fn map_pargs_errors(error: ValError) -> ValError {
    match error {
        ValError::LineErrors(line_errors) => {
            let line_errors = line_errors
                .into_iter()
                .map(|e| match e.kind {
                    ErrorKind::Missing => e.with_kind(ErrorKind::MissingPositionalArgument),
                    ErrorKind::TooLong {
                        max_length,
                        input_length,
                    } => e.with_kind(ErrorKind::UnexpectedPositionalArguments {
                        unexpected_count: input_length - max_length,
                    }),
                    _ => e,
                })
                .collect();
            ValError::LineErrors(line_errors)
        }
        internal_error => internal_error,
    }
}

fn map_kwargs_errors(error: ValError) -> ValError {
    match error {
        ValError::LineErrors(line_errors) => {
            let line_errors = line_errors
                .into_iter()
                .map(|e| match e.kind {
                    ErrorKind::Missing => e.with_kind(ErrorKind::MissingKeywordArgument),
                    ErrorKind::ExtraForbidden => e.with_kind(ErrorKind::UnexpectedKeywordArgument),
                    _ => e,
                })
                .collect();
            ValError::LineErrors(line_errors)
        }
        internal_error => internal_error,
    }
}
