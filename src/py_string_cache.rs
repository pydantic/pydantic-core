use std::collections::HashMap;
use std::hash::{BuildHasher, BuildHasherDefault, Hasher};
use std::sync::{Arc, Mutex};

use pyo3::once_cell::GILOnceCell;
use pyo3::prelude::*;
use pyo3::types::PyString;

use ahash::RandomState;
use nohash_hasher::BuildNoHashHasher;

type PyStringCache = Arc<Mutex<(RandomState, HashMap<u64, Py<PyString>, BuildNoHashHasher<u64>>)>>;
static PY_STRING_CACHE: GILOnceCell<PyStringCache> = GILOnceCell::new();

const LENGTH_LIMIT: usize = 63;
// gives max cache size of ~1MB
const MAX_ITEMS: usize = 16_000;

/// Creating PyStrings from a &str or String is somewhat expensive, so we cache short strings, this can make a 10%
/// improvement to performance where we're creating lots of pystrings from rust strings, e.g. JSON parsing.
/// Some scenarios might not want this, we could have a compile time option to disable it.
pub fn make_py_string<'py>(py: Python<'py>, s: &str) -> &'py PyString {
    if s.len() > LENGTH_LIMIT {
        return PyString::new(py, s);
    }

    let cache = PY_STRING_CACHE.get_or_init(py, create_cache);

    let (random_state, hashmap) = &mut *cache.lock().expect("Failed to acquire PY_STRING_CACHE lock");

    let mut hasher = random_state.build_hasher();
    hasher.write(s.as_bytes());
    let key = hasher.finish();

    if let Some(py_string) = hashmap.get(&key) {
        py_string.clone_ref(py).into_ref(py)
    } else {
        if hashmap.len() >= MAX_ITEMS {
            let keys: Vec<_> = hashmap.keys().take(200).cloned().collect();
            for k in keys {
                hashmap.remove(&k);
            }
        }
        let py_string = PyString::new(py, s);
        hashmap.insert(key, py_string.into_py(py));
        py_string
    }
}

fn create_cache() -> PyStringCache {
    let random_state = RandomState::new();
    let cache = HashMap::with_capacity_and_hasher(10, BuildHasherDefault::default());
    Arc::new(Mutex::new((random_state, cache)))
}
