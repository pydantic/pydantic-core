use ahash::AHashMap;

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
    counts: Option<AHashMap<RecursionKey, u16>>,
    depth: u16,
}

// A hard limit to avoid stack overflows when rampant recursion occurs
pub const RECURSION_GUARD_LIMIT: u16 = if cfg!(any(target_family = "wasm", all(windows, PyPy))) {
    // wasm and windows PyPy have very limited stack sizes
    50
} else if cfg!(any(PyPy, windows)) {
    // PyPy and Windows in general have more restricted stack space
    100
} else {
    255
};

// Maximum number of times we see the same object during serialization before we error
// In theory we should only need to visit the same object twice (in the case of duck typed serialization)
pub const CYCLE_LIMIT: u16 = 2;

impl RecursionGuard {
    fn visit(obj_id: usize, node_id: usize, counts: &mut AHashMap<(usize, usize), u16>) -> bool {
        let key = (obj_id, node_id);
        let entry = counts.entry(key);
        match entry {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let count = entry.get_mut();
                *count += 1;
                *count >= CYCLE_LIMIT
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(1);
                false
            }
        }
    }

    // insert a new id into the set, return whether we exceeded the cycle limit
    pub fn contains_or_insert(&mut self, obj_id: usize, node_id: usize) -> bool {
        match self.counts {
            Some(ref mut counts) => Self::visit(obj_id, node_id, counts),
            None => {
                let mut counts = AHashMap::new();
                Self::visit(obj_id, node_id, &mut counts);
                self.counts = Some(counts);
                false
            }
        }
    }

    pub fn incr_depth(&mut self) -> bool {
        self.depth += 1;
        self.depth >= RECURSION_GUARD_LIMIT
    }

    pub fn decr_depth(&mut self) {
        self.depth -= 1;
    }

    pub fn remove(&mut self, obj_id: usize, node_id: usize) {
        match self.counts {
            Some(ref mut counts) => {
                // decrease the count and pop the key if it's zero
                let key = (obj_id, node_id);
                let entry = counts.entry(key);
                match entry {
                    std::collections::hash_map::Entry::Occupied(mut entry) => {
                        let count = entry.get_mut();
                        *count -= 1;
                        if *count == 0 {
                            entry.remove();
                        }
                    }
                    std::collections::hash_map::Entry::Vacant(_) => {
                        panic!("RecursionGuard::remove called on a key that was not present: {key:?}")
                    }
                };
                if counts.is_empty() {
                    self.counts = None;
                }
            }
            None => panic!("RecursionGuard::remove called on a RecursionGuard with no active counts"),
        };
    }
}
