use std::fmt;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::build_tools::{is_strict, py_error, schema_or_config, SchemaDict};
use crate::errors::{ErrorKind, ValError, ValLineError, ValResult};
use crate::input::{GenericMapping, Input};
use crate::lookup_key::LookupKey;
use crate::recursion_guard::RecursionGuard;

use super::{build_validator, BuildContext, BuildValidator, CombinedValidator, Extra, Validator};

#[derive(Debug, Clone)]
pub struct UnionValidator {
    choices: Vec<CombinedValidator>,
    strict: bool,
    name: String,
}

impl BuildValidator for UnionValidator {
    const EXPECTED_TYPE: &'static str = "union";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let choices: Vec<CombinedValidator> = schema
            .get_as_req::<&PyList>("choices")?
            .iter()
            .map(|choice| build_validator(choice, config, build_context).map(|result| result.0))
            .collect::<PyResult<Vec<CombinedValidator>>>()?;

        let descr = choices.iter().map(|v| v.get_name()).collect::<Vec<_>>().join(",");

        Ok(Self {
            choices,
            strict: is_strict(schema, config)?,
            name: format!("{}[{}]", Self::EXPECTED_TYPE, descr),
        }
        .into())
    }
}

impl Validator for UnionValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        if extra.strict.unwrap_or(self.strict) {
            let mut errors: Vec<ValLineError> = Vec::with_capacity(self.choices.len());
            let strict_strict = extra.as_strict();

            for validator in &self.choices {
                let line_errors = match validator.validate(py, input, &strict_strict, slots, recursion_guard) {
                    Err(ValError::LineErrors(line_errors)) => line_errors,
                    otherwise => return otherwise,
                };

                errors.extend(
                    line_errors
                        .into_iter()
                        .map(|err| err.with_outer_location(validator.get_name().into())),
                );
            }

            Err(ValError::LineErrors(errors))
        } else {
            // 1st pass: check if the value is an exact instance of one of the Union types,
            // e.g. use validate in strict mode
            let strict_strict = extra.as_strict();
            if let Some(res) = self
                .choices
                .iter()
                .map(|validator| validator.validate(py, input, &strict_strict, slots, recursion_guard))
                .find(ValResult::is_ok)
            {
                return res;
            }

            let mut errors: Vec<ValLineError> = Vec::with_capacity(self.choices.len());

            // 2nd pass: check if the value can be coerced into one of the Union types, e.g. use validate
            for validator in &self.choices {
                let line_errors = match validator.validate(py, input, extra, slots, recursion_guard) {
                    Err(ValError::LineErrors(line_errors)) => line_errors,
                    success => return success,
                };

                errors.extend(
                    line_errors
                        .into_iter()
                        .map(|err| err.with_outer_location(validator.get_name().into())),
                );
            }

            Err(ValError::LineErrors(errors))
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn complete(&mut self, build_context: &BuildContext) -> PyResult<()> {
        self.choices.iter_mut().try_for_each(|v| v.complete(build_context))
    }
}

#[derive(Debug, Clone)]
struct TaggedUnionChoice {
    lookup_key: LookupKey,
    tag: String,
    validator: CombinedValidator,
}

impl fmt::Display for TaggedUnionChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})={}", self.lookup_key, self.tag)
    }
}

impl TaggedUnionChoice {
    fn from_py(tagged_union: &PyAny, config: Option<&PyDict>, build_context: &mut BuildContext) -> PyResult<Self> {
        let tagged_union: &PyDict = tagged_union.extract()?;
        let py = tagged_union.py();
        let lookup_key = match LookupKey::from_py(py, tagged_union, None, "tag_key", "tag_keys")? {
            Some(lookup_key) => lookup_key,
            None => return py_error!("'tag_key' or 'tag_keys' must be set on tagged union choices"),
        };
        Ok(Self {
            lookup_key,
            tag: tagged_union.get_as_req("tag")?,
            validator: build_validator(tagged_union.get_as_req("schema")?, config, build_context)?.0,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TaggedUnionValidator {
    choices: Vec<TaggedUnionChoice>,
    from_attributes: bool,
    strict: bool,
    name: String,
    tags_repr: String,
}

impl BuildValidator for TaggedUnionValidator {
    const EXPECTED_TYPE: &'static str = "tagged-union";

    fn build(
        schema: &PyDict,
        config: Option<&PyDict>,
        build_context: &mut BuildContext,
    ) -> PyResult<CombinedValidator> {
        let choices = schema
            .get_as_req::<&PyList>("choices")?
            .iter()
            .map(|choice| TaggedUnionChoice::from_py(choice, config, build_context))
            .collect::<PyResult<Vec<TaggedUnionChoice>>>()?;

        let tags_repr = choices.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", ");
        let descr = choices
            .iter()
            .map(|c| c.validator.get_name())
            .collect::<Vec<_>>()
            .join(",");
        let from_attributes =
            schema_or_config(schema, config, "from_attributes", "typed_dict_from_attributes")?.unwrap_or(false);

        Ok(Self {
            choices,
            from_attributes,
            strict: is_strict(schema, config)?,
            name: format!("{}[{}]", Self::EXPECTED_TYPE, descr),
            tags_repr,
        }
        .into())
    }
}

impl Validator for TaggedUnionValidator {
    fn validate<'s, 'data>(
        &'s self,
        py: Python<'data>,
        input: &'data impl Input<'data>,
        extra: &Extra,
        slots: &'data [CombinedValidator],
        recursion_guard: &'s mut RecursionGuard,
    ) -> ValResult<'data, PyObject> {
        let dict = input.typed_dict(self.from_attributes, !self.strict)?;

        for choice in &self.choices {
            // note all these methods return PyResult<Option<(data, data)>>, the outer Err is just for
            // errors when getting attributes which should be returned straight away,
            let d = match dict {
                GenericMapping::PyDict(d) => choice.lookup_key.py_get_item(d)?,
                GenericMapping::PyGetAttr(d) => choice.lookup_key.py_get_attr(d)?,
                GenericMapping::JsonObject(d) => choice.lookup_key.json_get(d)?,
            };

            if let Some((_, value)) = d {
                // if value is not a string, we don't raise an error, but continue to the next
                if let Ok(value_string) = value.extract::<String>() {
                    if value_string == choice.tag {
                        return choice.validator.validate(py, input, extra, slots, recursion_guard);
                    }
                }
            }
        }
        Err(ValError::new(
            ErrorKind::UnionTagNotFound {
                tags: self.tags_repr.clone(),
            },
            input,
        ))
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn complete(&mut self, build_context: &BuildContext) -> PyResult<()> {
        self.choices
            .iter_mut()
            .try_for_each(|c| c.validator.complete(build_context))
    }
}
