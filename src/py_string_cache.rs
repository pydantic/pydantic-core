use std::hash::{BuildHasher, Hasher};
use std::sync::{Arc, Mutex};

use pyo3::once_cell::GILOnceCell;
use pyo3::prelude::*;
use pyo3::types::PyString;

use ahash::RandomState;
use associative_cache::*;

type PyStringCache = Arc<
    Mutex<(
        RandomState,
        AssociativeCache<u64, Py<PyString>, Capacity8192, HashDirectMapped, RoundRobinReplacement>,
    )>,
>;
static PY_STRING_CACHE: GILOnceCell<PyStringCache> = GILOnceCell::new();

const LENGTH_LIMIT: usize = 63;

pub fn make_py_string<'py>(py: Python<'py>, s: &str) -> &'py PyString {
    if s.len() > LENGTH_LIMIT {
        return PyString::new(py, s);
    }

    let cache = PY_STRING_CACHE.get_or_init(py, || {
        let random_state = RandomState::new();
        let cache = AssociativeCache::default();
        Arc::new(Mutex::new((random_state, cache)))
    });
    let (random_state, cache) = &mut *cache.lock().expect("Failed to acquire PY_STRING_CACHE lock");

    let mut hasher = random_state.build_hasher();
    hasher.write(s.as_bytes());
    let hash = hasher.finish();

    let py_string = cache
        .entry(&hash)
        .or_insert_with(|| hash, || PyString::intern(py, s).into_py(py));
    py_string.clone_ref(py).into_ref(py)
}
