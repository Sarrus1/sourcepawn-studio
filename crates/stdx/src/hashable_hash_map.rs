use std::hash::{Hash, Hasher};

use fxhash::{FxHashMap, FxHashSet};

// FIXME: Hopefully there is a way to avoid Hashable Hashmaps?
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HashableHashMap<K: Hash + Eq, V: Eq> {
    pub map: FxHashMap<K, V>,
}

impl<K: Hash + Ord, V: Hash + Eq> Hash for HashableHashMap<K, V> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashableHashSet<K: Hash + Eq> {
    pub set: FxHashSet<K>,
}

impl<K: Hash + Ord> Hash for HashableHashSet<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Collect and sort the keys to ensure consistent order
        let mut pairs: Vec<_> = self.set.iter().collect();
        pairs.sort();
        pairs.iter().for_each(|k| k.hash(state));
    }
}

impl<K: Hash + Eq> Default for HashableHashSet<K> {
    fn default() -> Self {
        Self {
            set: FxHashSet::default(),
        }
    }
}
