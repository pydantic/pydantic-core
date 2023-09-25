/// Definition / reference management
/// Our definitions system is very similar to json schema's: there's ref strings and a definitions section
/// Unlike json schema we let you put definitions inline, not just in a single '#/$defs/' block or similar.
/// We use DefinitionsBuilder to collect the references / definitions into a single vector
/// and then get a definition from a reference using an integer id (just for performance of not using a HashMap)
use std::collections::hash_map::Entry;

use pyo3::{prelude::*, PyTraverseError, PyVisit};

use ahash::AHashMap;

use crate::{build_tools::py_schema_err, py_gc::PyGcTraverse};

// An integer id for the reference
pub type ReferenceId = usize;

/// Definitions are validators and serializers that are
/// shared by reference.
/// They come into play whenever there is recursion, e.g.
/// if you have validators A -> B -> A then A will be shared
/// by reference so that the SchemaValidator itself can own it.
/// These primarily get used by DefinitionRefValidator and DefinitionRefSerializer,
/// other validators / serializers primarily pass them around without interacting with them.
/// They get indexed by a ReferenceId, which are integer identifiers
/// that are handed out and managed by DefinitionsBuilder when the Schema{Validator,Serializer}
/// gets build.
pub type Definitions<T> = [Definition<T>];

#[derive(Clone, Debug)]
struct DefinitionSlot<T> {
    id: ReferenceId,
    value: Option<T>,
    recursive: bool,
}

#[derive(Clone)]
pub struct Definition<T> {
    pub value: T,
    pub recursive: bool,
}

impl<T: PyGcTraverse> PyGcTraverse for Definition<T> {
    fn py_gc_traverse(&self, visit: &PyVisit<'_>) -> Result<(), PyTraverseError> {
        self.value.py_gc_traverse(visit)
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Definition<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

#[derive(Clone, Debug)]
pub struct DefinitionsBuilder<T> {
    definitions: AHashMap<String, DefinitionSlot<T>>,
    in_flight_definitions: AHashMap<ReferenceId, bool>,
}

impl<T: Clone + std::fmt::Debug> DefinitionsBuilder<T> {
    pub fn new() -> Self {
        Self {
            definitions: AHashMap::new(),
            in_flight_definitions: AHashMap::new(),
        }
    }

    /// Get a ReferenceId for the given reference string.
    // This ReferenceId can later be used to retrieve a definition
    pub fn get_reference_id(&mut self, reference: &str) -> ReferenceId {
        let next_id = self.definitions.len();
        // We either need a String copy or two hashmap lookups
        // Neither is better than the other
        // We opted for the easier outward facing API
        let id = match self.definitions.entry(reference.to_string()) {
            Entry::Occupied(entry) => entry.get().id,
            Entry::Vacant(entry) => {
                entry.insert(DefinitionSlot {
                    id: next_id,
                    value: None,
                    recursive: false,
                });
                next_id
            }
        };
        // If this definition is currently being built, then it's recursive
        if let Some(recursive) = self.in_flight_definitions.get_mut(&id) {
            *recursive = true;
        }
        id
    }

    /// Add a definition
    pub fn build_definition(
        &mut self,
        reference: String,
        constructor: impl FnOnce(&mut Self) -> PyResult<T>,
    ) -> PyResult<()> {
        let next_id = self.definitions.len();
        let id = match self.definitions.entry(reference.clone()) {
            Entry::Occupied(entry) => {
                let entry = entry.into_mut();
                if entry.value.is_some() {
                    return py_schema_err!("Duplicate ref: `{}`", reference);
                }
                entry
            }
            Entry::Vacant(entry) => entry.insert(DefinitionSlot {
                id: next_id,
                value: None,
                recursive: false,
            }),
        }
        .id;
        self.in_flight_definitions.insert(id, false);
        let value = constructor(self)?;
        // can unwrap because the entry was just built above
        let slot = self.definitions.get_mut(&reference).unwrap();
        slot.value = Some(value);
        slot.recursive = self.in_flight_definitions.remove(&id).unwrap();
        Ok(())
    }

    /// Retrieve an item definition using a ReferenceId
    /// If the definition doesn't yet exist (as happens in recursive types) then we create it
    /// At the end (in finish()) we check that there are no undefined definitions
    pub fn get_definition(&self, reference_id: ReferenceId) -> PyResult<&T> {
        let (reference, def) = match self.definitions.iter().find(|(_, def)| def.id == reference_id) {
            Some(v) => v,
            None => return py_schema_err!("Definitions error: no definition for ReferenceId `{}`", reference_id),
        };
        match def.value.as_ref() {
            Some(v) => Ok(v),
            None => py_schema_err!(
                "Definitions error: attempted to use `{}` before it was filled",
                reference
            ),
        }
    }

    /// Consume this Definitions into a vector of items, indexed by each items ReferenceId
    pub fn finish(self) -> PyResult<Vec<Definition<T>>> {
        // We need to create a vec of defs according to the order in their ids
        let mut defs: Vec<(usize, Definition<T>)> = Vec::new();
        for (reference, def) in self.definitions {
            match def.value {
                None => return py_schema_err!("Definitions error: definition {} was never filled", reference),
                Some(value) => defs.push((
                    def.id,
                    Definition {
                        value,
                        recursive: def.recursive,
                    },
                )),
            }
        }
        defs.sort_by_key(|(id, _)| *id);
        Ok(defs.into_iter().map(|(_, v)| v).collect())
    }
}
