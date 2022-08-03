use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::{Arc, Mutex};

use pyo3::once_cell::GILOnceCell;
use pyo3::prelude::*;
use pyo3::types::PyString;

use ahash::AHasher;
use nohash_hasher::BuildNoHashHasher;

type PyStringCache = (
    u128,
    u128,
    Arc<Mutex<HashMap<u64, Py<PyString>, BuildNoHashHasher<u64>>>>,
);
static PY_STRING_CACHE: GILOnceCell<PyStringCache> = GILOnceCell::new();

const LENGTH_LIMIT: usize = 63;
// gives max cache size of ~2MB
const MAX_ITEMS: usize = 32_000;

pub fn make_py_string<'py>(py: Python<'py>, s: &str) -> &'py PyString {
    if s.len() > LENGTH_LIMIT {
        return PyString::new(py, s);
    }

    let (key1, key2, cache) = PY_STRING_CACHE.get_or_init(py, || {
        let hashmap = HashMap::with_capacity_and_hasher(100, BuildHasherDefault::default());
        (123, 321, Arc::new(Mutex::new(hashmap)))
    });

    let mut hashmap = cache.lock().expect("Failed to acquire PY_STRING_CACHE lock");

    let mut hasher = AHasher::new_with_keys(*key1, *key2);
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
