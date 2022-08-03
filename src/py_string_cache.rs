use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::{Arc, Mutex};

use pyo3::once_cell::GILOnceCell;
use pyo3::prelude::*;
use pyo3::types::PyString;

use ahash::AHasher;
use nohash_hasher::BuildNoHashHasher;

type PyStringCache = Arc<Mutex<(AHasher, HashMap<u64, Py<PyString>, BuildNoHashHasher<u64>>)>>;
static PY_STRING_CACHE: GILOnceCell<PyStringCache> = GILOnceCell::new();

const LENGTH_LIMIT: usize = 63;
// gives max cache size of ~2MB
const MAX_ITEMS: usize = 32_000;

pub fn make_py_string<'py>(py: Python<'py>, s: &str) -> &'py PyString {
    if s.len() > LENGTH_LIMIT {
        return PyString::new(py, s);
    }

    let cache = PY_STRING_CACHE.get_or_init(py, || {
        let hasher = AHasher::default();
        let hashmap = HashMap::with_capacity_and_hasher(100, BuildHasherDefault::default());
        Arc::new(Mutex::new((hasher, hashmap)))
    });

    let mut cache = cache.lock().expect("Failed to acquire PY_STRING_CACHE lock");
    let (hasher, hashmap) = &mut *cache;

    hasher.write(s.as_bytes());
    let key = hasher.finish();

    if let Some(py_string) = hashmap.get(&key) {
        py_string.clone_ref(py).into_ref(py)
    } else {
        let py_string = PyString::new(py, s);
        hashmap.insert(key, py_string.into_py(py));
        if hashmap.len() > MAX_ITEMS {
            let keys: Vec<_> = hashmap.keys().take(1000).cloned().collect();
            for k in keys {
                hashmap.remove(&k);
            }
        }
        py_string
    }
}
