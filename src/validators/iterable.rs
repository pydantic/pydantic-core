use std::sync::Arc;

use jiter::JsonValue;
use pyo3::types::PyDict;
use pyo3::{intern, prelude::*, IntoPyObjectExt};

use crate::errors::ValResult;
use crate::input::{
    validate_iter_to_vec, GenericIterator, GenericJsonIterator, GenericPyIterator, Input, MaxLengthCheck,
};
use crate::tools::SchemaDict;
use crate::validators::any::AnyValidator;
use crate::validators::generator::GeneratorValidator;
use crate::validators::list::min_length_check;

use super::list::get_items_schema;
use super::{BuildValidator, CombinedValidator, DefinitionsBuilder, ValidationState, Validator};

#[derive(Debug, Clone)]
pub struct IterableValidator {
    item_validator: Option<Arc<CombinedValidator>>,
    min_length: Option<usize>,
    max_length: Option<usize>,
    name: String,
}

impl BuildValidator for IterableValidator {
    const EXPECTED_TYPE: &'static str = "iterable";

    fn build(
        schema: &Bound<'_, PyDict>,
        config: Option<&Bound<'_, PyDict>>,
        definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        // TODO: in Pydantic V3 default will be lazy=False
        let lazy_iterable: bool = schema.get_as(intern!(schema.py(), "lazy"))?.unwrap_or(true);

        if lazy_iterable {
            // lazy iterable is equivalent to generator, for backwards compatibility
            return GeneratorValidator::build(schema, config, definitions);
        }

        let item_validator = get_items_schema(schema, config, definitions)?.map(Arc::new);
        let name = match item_validator {
            Some(ref v) => format!("{}[{}]", Self::EXPECTED_TYPE, v.get_name()),
            None => format!("{}[any]", Self::EXPECTED_TYPE),
        };
        Ok(Self {
            item_validator,
            name,
            min_length: schema.get_as(pyo3::intern!(schema.py(), "min_length"))?,
            max_length: schema.get_as(pyo3::intern!(schema.py(), "max_length"))?,
        }
        .into())
    }
}

impl_py_gc_traverse!(IterableValidator { item_validator });

impl Validator for IterableValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<Py<PyAny>> {
        // this validator does not yet support partial validation, disable it to avoid incorrect results
        state.allow_partial = false.into();

        let iterator = input.validate_iter()?;

        let item_validator = self
            .item_validator
            .as_deref()
            .unwrap_or(&CombinedValidator::Any(AnyValidator));

        let max_length_check = MaxLengthCheck::new(self.max_length, "Iterable", input, None);
        let vec = match iterator {
            GenericIterator::PyIterator(iter) => validate_iter_to_vec(
                py,
                IterWithPy { py, iter },
                0,
                max_length_check,
                item_validator,
                state,
                false,
            )?,
            GenericIterator::JsonArray(iter) => validate_iter_to_vec(
                py,
                IterWithPy { py, iter },
                0,
                max_length_check,
                item_validator,
                state,
                false,
            )?,
        };

        min_length_check!(input, "Iterable", self.min_length, vec);

        vec.into_py_any(py).map_err(Into::into)
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

struct IterWithPy<'py, I> {
    py: Python<'py>,
    iter: I,
}

impl<'py> Iterator for IterWithPy<'py, GenericPyIterator> {
    type Item = PyResult<Bound<'py, PyAny>>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.iter.next(self.py).transpose()?.map(|(v, _)| v))
    }
}

impl<'j> Iterator for IterWithPy<'_, GenericJsonIterator<'j>> {
    type Item = PyResult<JsonValue<'j>>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.iter.next(self.py).transpose()?.map(|(v, _)| v.clone()))
    }
}
