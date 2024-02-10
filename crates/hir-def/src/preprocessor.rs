use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use fxhash::FxHashMap;
use preprocessor::{Macro, PreprocessingResult, SourcepawnPreprocessor};
use vfs::{AnchoredPath, FileId};

use crate::DefDatabase;

// FIXME: Everything about this is bad.
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

pub(crate) fn preprocess_file_query(
    db: &dyn DefDatabase,
    file_id: FileId,
) -> Arc<PreprocessingResult> {
    // FIXME: Handle cyclic dependencies
    // FIXME: Handle errors
    let root_file_id = db.projet_subgraph(file_id).unwrap().root.file_id;
    let res = db.preprocess_file_inner(root_file_id, HashableHashMap::default());

    res.get(&file_id).unwrap().clone()
}

// FIXME: When preprocessing subfiles, we do not know the macros of the parent file.
// Solutions:
// 1. Make a struct that holds the macros that is always equal and always has the same hash?
// 2. Get all the parents's macros and pass them to the subfile's preprocessing function?
pub(crate) fn _preprocess_file_query(
    db: &dyn DefDatabase,
    file_id: FileId,
    macros: HashableHashMap<String, Macro>,
) -> Arc<FxHashMap<FileId, Arc<PreprocessingResult>>> {
    // FIXME: Handle cyclic dependencies
    let text = db.file_text(file_id);
    let mut results: FxHashMap<FileId, Arc<PreprocessingResult>> = FxHashMap::default();
    let mut preprocessor = SourcepawnPreprocessor::new(file_id, &text);
    preprocessor.set_macros(macros.map);
    let res = preprocessor
        .preprocess_input(
            &mut (|macros: &mut FxHashMap<String, Macro>,
                   path: String,
                   file_id: FileId,
                   quoted: bool| {
                let mut inc_file_id = None;
                if quoted {
                    inc_file_id = db.resolve_path(AnchoredPath::new(file_id, &path));
                };
                if inc_file_id.is_none() {
                    inc_file_id = db.resolve_path_relative_to_roots(&path);
                }
                if let Some(inc_file_id) = inc_file_id {
                    let map = HashableHashMap {
                        map: macros.clone(),
                    };
                    let res = db.preprocess_file_inner(inc_file_id, map);
                    results.extend(res.as_ref().clone()); // FIXME: This is bad
                    macros.extend(res.as_ref().get(&inc_file_id).unwrap().macros().clone());
                }

                Ok(())
            }),
        )
        .unwrap()
        .result();
    results.insert(file_id, res.into());

    results.into()
}
