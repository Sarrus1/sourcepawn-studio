use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use fxhash::{FxHashMap, FxHashSet};
use preprocessor::{Macro, PreprocessingResult, SourcepawnPreprocessor};
use vfs::{AnchoredPath, FileId};

use crate::{db::infer_include_ext, DefDatabase};

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

pub(crate) fn preprocess_file_query(
    db: &dyn DefDatabase,
    file_id: FileId,
) -> Arc<PreprocessingResult> {
    // FIXME: Handle errors
    let root_file_id = db.projet_subgraph(file_id).unwrap().root.file_id;
    let res = db.preprocess_file_inner(
        root_file_id,
        HashableHashMap::default(),
        HashableHashSet::default(),
    );

    res.get(&file_id).unwrap().clone()
}

pub(crate) fn _preprocess_file_query(
    db: &dyn DefDatabase,
    file_id: FileId,
    macros: HashableHashMap<String, Macro>,
    mut being_preprocessed: HashableHashSet<FileId>,
) -> Arc<FxHashMap<FileId, Arc<PreprocessingResult>>> {
    being_preprocessed.set.insert(file_id);
    let text = db.file_text(file_id);
    let mut results: FxHashMap<FileId, Arc<PreprocessingResult>> = FxHashMap::default();
    let mut preprocessor = SourcepawnPreprocessor::new(file_id, &text);
    preprocessor.set_macros(macros.map);
    let res = preprocessor
        .preprocess_input(
            &mut (|macros: &mut FxHashMap<String, Macro>,
                   mut path: String,
                   file_id: FileId,
                   quoted: bool| {
                let mut inc_file_id = None;
                infer_include_ext(&mut path);
                if quoted {
                    inc_file_id = db.resolve_path(AnchoredPath::new(file_id, &path));
                };
                if inc_file_id.is_none() {
                    inc_file_id = db.resolve_path_relative_to_roots(&path);
                }
                let inc_file_id =
                    inc_file_id.ok_or_else(|| anyhow::anyhow!("Include not found"))?;
                if being_preprocessed.set.contains(&inc_file_id) {
                    // Avoid cyclic deps
                    return Ok(());
                }
                let map = HashableHashMap {
                    map: macros.clone(),
                };
                let res = db.preprocess_file_inner(inc_file_id, map, being_preprocessed.clone());
                results.extend(res.as_ref().clone()); //FIXME: Maybe find a way to avoid this clone?
                macros.extend(res.as_ref().get(&inc_file_id).unwrap().macros().clone());

                Ok(())
            }),
        )
        .unwrap()
        .result();
    results.insert(file_id, res.into());

    results.into()
}
