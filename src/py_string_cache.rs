use std::hash::Hasher;
use std::sync::{Arc, Mutex};

use pyo3::once_cell::GILOnceCell;
use pyo3::prelude::*;
use pyo3::types::PyString;

use ahash::AHasher;
use associative_cache::*;

type PyStringCache = (
    u128,
    u128,
    Arc<Mutex<AssociativeCache<u64, Py<PyString>, Capacity8192, HashDirectMapped, RoundRobinReplacement>>>,
);
static PY_STRING_CACHE: GILOnceCell<PyStringCache> = GILOnceCell::new();

const LENGTH_LIMIT: usize = 63;

pub fn make_py_string<'py>(py: Python<'py>, s: &str) -> &'py PyString {
    if s.len() > LENGTH_LIMIT {
        return PyString::new(py, s);
    }

    let (key1, key2, cache) = PY_STRING_CACHE.get_or_init(py, || {
        let cache = AssociativeCache::default();
        (123, 321, Arc::new(Mutex::new(cache)))
    });

    let mut hasher = AHasher::new_with_keys(*key1, *key2);
    hasher.write(s.as_bytes());
    let hash = hasher.finish();

    let mut cache = cache.lock().expect("Failed to acquire PY_STRING_CACHE lock");
    let py_string = cache
        .entry(&hash)
        .or_insert_with(|| hash, || PyString::new(py, s).into_py(py));
    py_string.clone_ref(py).into_ref(py)
}
