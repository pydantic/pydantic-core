use pyo3::once_cell::GILOnceCell;
use pyo3::prelude::*;
use pyo3::types::PyString;
use std::sync::{Arc, Mutex};

use ahash::AHashMap;

type PyStringCache = Arc<Mutex<AHashMap<String, Py<PyString>>>>;
static PY_STRING_CACHE: GILOnceCell<PyStringCache> = GILOnceCell::new();

const LENGTH_LIMIT: usize = 63;
// gives max cache size of ~2MB
const MAX_ITEMS: usize = 32_000;

pub fn make_py_string<'py>(py: Python<'py>, s: &str) -> &'py PyString {
    if s.len() > LENGTH_LIMIT {
        return PyString::new(py, s);
    }

    let cache = PY_STRING_CACHE.get_or_init(py, || Arc::new(Mutex::new(AHashMap::with_capacity(100))));

    let mut hashmap = cache.lock().expect("Failed to acquire PY_STRING_CACHE lock");
    if let Some(py_string) = hashmap.get(s) {
        py_string.clone_ref(py).into_ref(py)
    } else {
        if hashmap.len() >= MAX_ITEMS {
            let keys: Vec<_> = hashmap.keys().take(1000).cloned().collect();
            for k in keys {
                hashmap.remove(&k);
            }
        }
        let py_string = PyString::intern(py, s);
        hashmap.insert(s.to_string(), py_string.into_py(py));
        py_string
    }
}
