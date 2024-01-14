use ahash::AHashSet;
use std::hash::Hash;

type RecursionKey = (
    // Identifier for the input object, e.g. the id() of a Python dict
    usize,
    // Identifier for the node we are traversing, e.g. the validator's id
    // Generally only things that can be traversed multiple times, like a definition reference
    // need to use the recursion guard, and those things should already have a natural node id
    usize,
);

/// This is used to avoid cyclic references in input data causing recursive validation and a nasty segmentation fault.
/// It's used in `validators/definition` to detect when a reference is reused within itself.
#[derive(Debug, Clone, Default)]
pub struct RecursionGuard {
    ids: SmallContainer<RecursionKey>,
    // depth could be a hashmap {validator_id => depth} but for simplicity and performance it's easier to just
    // use one number for all validators
    depth: u8,
}

// A hard limit to avoid stack overflows when rampant recursion occurs
pub const RECURSION_GUARD_LIMIT: u8 = if cfg!(any(target_family = "wasm", all(windows, PyPy))) {
    // wasm and windows PyPy have very limited stack sizes
    49
} else if cfg!(any(PyPy, windows)) {
    // PyPy and Windows in general have more restricted stack space
    99
} else {
    255
};

impl RecursionGuard {
    // insert a new value
    // * return `None` if the array/set already had it in it
    // * return `Some(index)` if the array didn't have it in it and it was inserted
    pub fn contains_or_insert(&mut self, obj_id: usize, node_id: usize) -> Option<usize> {
        self.ids.contains_or_insert((obj_id, node_id))
    }

    // see #143 this is used as a backup in case the identity check recursion guard fails
    #[must_use]
    #[cfg(any(target_family = "wasm", windows, PyPy))]
    pub fn incr_depth(&mut self) -> bool {
        // use saturating_add as it's faster (since there's no error path)
        // and the RECURSION_GUARD_LIMIT check will be hit before it overflows
        debug_assert!(RECURSION_GUARD_LIMIT < 255);
        self.depth = self.depth.saturating_add(1);
        self.depth > RECURSION_GUARD_LIMIT
    }

    #[must_use]
    #[cfg(not(any(target_family = "wasm", windows, PyPy)))]
    pub fn incr_depth(&mut self) -> bool {
        debug_assert_eq!(RECURSION_GUARD_LIMIT, 255);
        // use checked_add to check if we've hit the limit
        if let Some(depth) = self.depth.checked_add(1) {
            self.depth = depth;
            false
        } else {
            true
        }
    }

    pub fn decr_depth(&mut self) {
        // for the same reason as incr_depth, use saturating_sub
        self.depth = self.depth.saturating_sub(1);
    }

    pub fn remove(&mut self, obj_id: usize, node_id: usize, index: usize) {
        self.ids.remove(&(obj_id, node_id), index);
    }
}

// trial and error suggests this is a good value, going higher causes array lookups to get significantly slower
const ARRAY_SIZE: usize = 16;

#[derive(Debug, Clone)]
enum SmallContainer<T> {
    Array([Option<T>; ARRAY_SIZE]),
    Set(AHashSet<T>),
}

impl<T: Copy> Default for SmallContainer<T> {
    fn default() -> Self {
        Self::Array([None; ARRAY_SIZE])
    }
}

impl<T: Eq + Hash + Clone> SmallContainer<T> {
    // insert a new value
    // * return `None` if the array/set already had it in it
    // * return `Some(index)` if the array didn't have it in it and it was inserted
    pub fn contains_or_insert(&mut self, v: T) -> Option<usize> {
        match self {
            Self::Array(array) => {
                let mut first_slot: Option<usize> = None;
                for (index, op_value) in array.iter().enumerate() {
                    if let Some(existing) = op_value {
                        if existing == &v {
                            return None;
                        }
                    } else {
                        first_slot = first_slot.or(Some(index));
                    }
                }
                if let Some(index) = first_slot {
                    array[index] = Some(v);
                    first_slot
                } else {
                    let mut set: AHashSet<T> = AHashSet::with_capacity(ARRAY_SIZE + 1);
                    for existing in array.iter_mut() {
                        set.insert(existing.take().unwrap());
                    }
                    set.insert(v);
                    *self = Self::Set(set);
                    // id doesn't matter here as we'll be removing from a set
                    Some(0)
                }
            }
            // https://doc.rust-lang.org/std/collections/struct.HashSet.html#method.insert
            // "If the set did not have this value present, `true` is returned."
            Self::Set(set) => {
                if set.insert(v) {
                    // again id doesn't matter here as we'll be removing from a set
                    Some(0)
                } else {
                    None
                }
            }
        }
    }

    pub fn remove(&mut self, v: &T, index: usize) {
        match self {
            Self::Array(array) => {
                array[index] = None;
            }
            Self::Set(set) => {
                set.remove(v);
            }
        }
    }
}
