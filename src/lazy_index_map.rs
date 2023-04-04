use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::{Eq, PartialEq};
use std::fmt::Debug;
use std::hash::Hash;
use std::slice::Iter as SliceIter;

use ahash::AHashMap;

#[derive(Debug, Clone, Default)]
pub struct LazyIndexMap<K, V> {
    vec: Vec<(K, V)>,
    map: RefCell<Option<AHashMap<K, usize>>>,
}

/// Like [IndexMap](https://docs.rs/indexmap/latest/indexmap/) but only builds the lookup map when it's needed.
impl<K, V> LazyIndexMap<K, V>
where
    K: Clone + Debug + Eq + Hash,
    V: Clone + Debug,
{
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            map: RefCell::new(None),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.vec.push((key, value))
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + PartialEq<Q>,
        Q: Hash + Eq,
    {
        let mut map = self.map.borrow_mut();
        if let Some(map) = map.as_ref() {
            if let Some(index) = map.get(key) {
                return Some(&self.vec[*index].1);
            }
        }
        if let Some((index, (k, v))) = self.vec.iter().enumerate().find(|(_, (k, _))| k == key) {
            if map.is_none() {
                *map = Some(AHashMap::with_capacity(self.vec.len()));
            }
            map.as_mut().unwrap().insert(k.clone(), index);
            Some(v)
        } else {
            None
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.vec.iter().map(|(k, _)| k)
    }

    pub fn iter(&self) -> SliceIter<'_, (K, V)> {
        self.vec.iter()
    }
}
