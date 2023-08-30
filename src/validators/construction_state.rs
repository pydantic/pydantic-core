use pyo3::types::PySet;

pub struct ConstructionState<'a> {
    pub fields_set: Option<&'a PySet>,
    pub recursive: bool,
}

impl<'a> ConstructionState<'a> {
    pub fn new(fields_set: Option<&'a PySet>, recursive: bool) -> Self {
        Self { fields_set, recursive }
    }
}
