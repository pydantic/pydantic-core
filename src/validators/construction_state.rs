use pyo3::types::PyAny;

use crate::definitions::Definitions;
use crate::input::InputType;

use super::CombinedValidator;

pub struct ConstructionState<'a> {
    pub fields_set: Option<&'a PyAny>,
    pub recursive: bool,
    pub mode: InputType,
    pub strict: bool,
    pub ultra_strict: bool,
    pub definitions: &'a Definitions<CombinedValidator>,
}

impl<'a> ConstructionState<'a> {
    pub fn new(
        fields_set: Option<&'a PyAny>,
        recursive: bool,
        mode: InputType,
        strict: bool,
        ultra_strict: bool,
        definitions: &'a Definitions<CombinedValidator>,
    ) -> Self {
        Self {
            fields_set,
            recursive,
            mode,
            strict,
            ultra_strict,
            definitions,
        }
    }
}
