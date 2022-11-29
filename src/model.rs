// use std::collections::hash_map::Entry;
// use std::collections::HashMap;
// use std::hash::BuildHasherDefault;

use pyo3::exceptions::PyAttributeError;
use pyo3::prelude::*;
use pyo3::types::{PySet, PyString};
use pyo3::{AsPyPointer, FromPyPointer, PyNativeType};

// use ahash::RandomState;
// use nohash_hasher::NoHashHasher;
use crate::build_tools::py_err;

// type IntHashBuilder = BuildHasherDefault<NoHashHasher<u64>>;
// type IntHashMap = HashMap<u64, usize, IntHashBuilder>;

struct ModelAttribute {
    key: Py<PyString>,
    value: PyObject,
    field_set: bool,
}

#[pyclass(subclass, module = "pydantic_core._pydantic_core")]
pub struct PydanticCoreModel {
    attributes: Vec<ModelAttribute>,
    // lookup: IntHashMap,
    // hash_builder: RandomState,
}

impl PydanticCoreModel {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            attributes: Vec::with_capacity(capacity),
            // lookup: IntHashMap::with_capacity_and_hasher(capacity, IntHashBuilder::default()),
            // hash_builder: RandomState::new(),
        }
    }

    pub fn set_item<V>(&mut self, key: &PyString, value: V, field_set: bool)
    where
        V: ToPyObject,
    {
        let py = key.py();
        self.attributes.push(ModelAttribute {
            key: key.into(),
            value: value.to_object(py),
            field_set,
        });

        // let str = key.to_str()?;
        // let hash = self.hash_builder.hash_one(str);
        //
        // match self.lookup.entry(hash) {
        //     Entry::Occupied(entry) => {
        //         let index = *entry.get();
        //         self.entries[index].value = value.to_object(py);
        //     }
        //     Entry::Vacant(entry) => {
        //         entry.insert(self.entries.len());
        //         self.entries.push(ModelAttribute {
        //             key: key.into(),
        //             value: value.to_object(py),
        //         });
        //     }
        // };
        // Ok(())
    }
}

#[pymethods]
impl PydanticCoreModel {
    fn __getattr__(self_: PyRef<Self>, py: Python, key: &PyString) -> PyResult<PyObject> {
        for attr in &self_.attributes {
            if attr.key.as_ref(py).eq(key).unwrap_or(false) {
                return Ok(attr.value.clone_ref(py));
            }
        }
        let t = unsafe { PyAny::from_owned_ptr(py, self_.as_ptr()) };
        py_err!(PyAttributeError; "'{:?}' has no attribute '{}'", t, key.to_str()?)
        // let str = key.to_str()?;
        // let hash = self.hash_builder.hash_one(str);
        //
        // match self.lookup.get(&hash) {
        //     Some(index) => {
        //         let entry = &self.entries[*index];
        //         Ok(entry.value.clone_ref(py))
        //     }
        //     None => Err(PyKeyError::new_err::<PyObject>(key.into())),
        // }
    }

    #[getter]
    fn __fields_set__<'py>(&self, py: Python<'py>) -> PyResult<&'py PySet> {
        let elements = self
            .attributes
            .iter()
            .filter(|attr| attr.field_set)
            .map(|attr| attr.key.as_ref(py) as &PyAny)
            .collect::<Vec<&PyAny>>();
        PySet::new(py, &elements)
    }
}
