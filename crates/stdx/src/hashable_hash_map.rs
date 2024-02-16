//! Hashable HashMap and HashSet

use std::hash::{Hash, Hasher};

use fxhash::{FxHashMap, FxHashSet};

/// Wrapper around `FxHashMap` that implements `Hash`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashableHashMap<K: Hash + Eq, V: Eq> {
    map: FxHashMap<K, V>,
}

impl<K: Hash + Eq + Clone, V: Eq + Clone> HashableHashMap<K, V> {
    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.map.insert(key, value)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.map.remove(key)
    }

    pub fn extend(&mut self, other: HashableHashMap<K, V>) {
        self.map.extend(other.map)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map.iter()
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.map.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.map.values()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn clear(&mut self) {
        self.map.clear()
    }

    pub fn to_map(&self) -> FxHashMap<K, V> {
        self.map.clone()
    }
}

impl<K: Hash + Ord + Clone, V: Hash + Eq + Clone> Hash for HashableHashMap<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Collect and sort the keys to ensure consistent order
        let mut pairs: Vec<_> = self.map.iter().collect();
        pairs.sort_by(|a, b| a.0.cmp(b.0));

        // Hash each key-value pair
        for (key, value) in pairs {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl<K: Hash + Ord, V: Hash + Eq> From<FxHashMap<K, V>> for HashableHashMap<K, V> {
    fn from(map: FxHashMap<K, V>) -> Self {
        Self { map }
    }
}

impl<K: Hash + Eq, V: Eq> Default for HashableHashMap<K, V> {
    fn default() -> Self {
        Self {
            map: FxHashMap::default(),
        }
    }
}

/// Wrapper around `FxHashSet` that implements `Hash`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashableHashSet<K: Hash + Eq> {
    set: FxHashSet<K>,
}

impl<K: Hash + Eq> HashableHashSet<K> {
    pub fn insert(&mut self, key: K) -> bool {
        self.set.insert(key)
    }

    pub fn remove(&mut self, key: &K) -> bool {
        self.set.remove(key)
    }

    pub fn contains(&self, key: &K) -> bool {
        self.set.contains(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = &K> {
        self.set.iter()
    }

    pub fn len(&self) -> usize {
        self.set.len()
    }

    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    pub fn clear(&mut self) {
        self.set.clear()
    }
}

impl<K: Hash + Ord> Hash for HashableHashSet<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Collect and sort the keys to ensure consistent order
        let mut pairs: Vec<_> = self.set.iter().collect();
        pairs.sort();
        pairs.iter().for_each(|k| k.hash(state));
    }
}

impl<K: Hash + Eq> From<FxHashSet<K>> for HashableHashSet<K> {
    fn from(set: FxHashSet<K>) -> Self {
        Self { set }
    }
}

impl<K: Hash + Eq> Default for HashableHashSet<K> {
    fn default() -> Self {
        Self {
            set: FxHashSet::default(),
        }
    }
}
