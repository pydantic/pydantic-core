use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

use pyo3::prelude::*;
use pyo3::types::PyString;

use ahash::RandomState;
use nohash_hasher::NoHashHasher;
use pyo3::exceptions::PyKeyError;
use pyo3::PyNativeType;

type IntHashBuilder = BuildHasherDefault<NoHashHasher<u64>>;
type IntHashMap = HashMap<u64, usize, IntHashBuilder>;

struct FastEntry {
    key: Py<PyString>,
    value: PyObject,
}

#[pyclass(mapping, module = "pydantic_core._pydantic_core")]
pub struct FastDict {
    entries: Vec<FastEntry>,
    lookup: IntHashMap,
    hash_builder: RandomState,
}

impl FastDict {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            lookup: IntHashMap::with_capacity_and_hasher(capacity, IntHashBuilder::default()),
            hash_builder: RandomState::new(),
        }
    }

    pub fn set_item<V>(&mut self, key: &PyString, value: V) -> PyResult<()>
    where
        V: ToPyObject,
    {
        let py = key.py();
        let str = key.to_str()?;
        let hash = self.hash_builder.hash_one(str);

        match self.lookup.entry(hash) {
            Entry::Occupied(entry) => {
                let index = *entry.get();
                self.entries[index].value = value.to_object(py);
            }
            Entry::Vacant(entry) => {
                entry.insert(self.entries.len());
                self.entries.push(FastEntry {
                    key: key.into(),
                    value: value.to_object(py),
                });
            }
        };
        Ok(())
    }
}

#[pymethods]
impl FastDict {
    fn getitem(&self, py: Python, key: &PyString) -> PyResult<PyObject> {
        let str = key.to_str()?;
        let hash = self.hash_builder.hash_one(str);

        match self.lookup.get(&hash) {
            Some(index) => {
                let entry = &self.entries[*index];
                Ok(entry.value.clone_ref(py))
            }
            None => Err(PyKeyError::new_err::<PyObject>(key.into())),
        }
    }

    fn get(&self, py: Python, key: &PyString) -> PyResult<PyObject> {
        let str = key.to_str()?;
        let hash = self.hash_builder.hash_one(str);

        match self.lookup.get(&hash) {
            Some(index) => {
                let entry = &self.entries[*index];
                Ok(entry.value.clone_ref(py))
            }
            None => Ok(py.None()),
        }
    }
}
