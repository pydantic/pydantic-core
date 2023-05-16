use std::cmp;
use std::hash;

use pyo3::basic::CompareOp;
use pyo3::prelude::*;

// We can't put a Py<PyAny> directly into a HashMap key
// So to be able to hold references to arbitrary Python objects in HashMap as keys
// we wrap them in a struct that gets the hash() when it receives the object from Python
// and then just echoes back that hash when called Rust needs to hash it
#[derive(Clone)]
pub struct HashedAny {
    pub value: Py<PyAny>,
    hash: isize,
}

impl<'source> FromPyObject<'source> for HashedAny {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        Ok(HashedAny {
            value: ob.into(),
            hash: ob.hash()?,
        })
    }
}

impl IntoPy<PyObject> for HashedAny {
    fn into_py(self, _py: Python<'_>) -> PyObject {
        self.value
    }
}

impl ToPyObject for HashedAny {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.value.to_object(py)
    }
}

impl hash::Hash for HashedAny {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write_isize(self.hash)
    }
}

impl cmp::PartialEq for HashedAny {
    fn eq(&self, other: &Self) -> bool {
        // This assumes that `self is other` implies `self == other`
        // Which is not necessarily true, e.g. for NaN, but is true in most cases
        // and there's a perf advantage to not calling into Python
        if self.value.is(&other.value) {
            return true;
        }
        Python::with_gil(|py| -> bool {
            self.value
                .as_ref(py)
                .rich_compare(other.value.as_ref(py), CompareOp::Eq)
                .unwrap()
                .is_true()
                .unwrap()
        })
    }
}

impl cmp::Eq for HashedAny {}

// Since we are only ever using isize values as keys
// and these values are already being hashed in Python land
// we don't need to re-hash them in Rust
// The following is a Hasher implementation that does no hashing

#[derive(Default, Clone, Copy)]
pub struct NoHashHasher {
    val: u64,
}

impl hash::Hasher for NoHashHasher {
    fn write(&mut self, _: &[u8]) {
        panic!("Invalid use of NoHashHasher")
    }

    fn write_isize(&mut self, n: isize) {
        self.val = n as u64;
    }

    fn finish(&self) -> u64 {
        self.val
    }
}

pub type HashedAnyMap<V> = std::collections::HashMap<HashedAny, V, std::hash::BuildHasherDefault<NoHashHasher>>;
