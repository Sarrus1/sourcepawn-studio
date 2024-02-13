use std::sync::Arc;

use base_db::{infer_include_ext, SourceDatabase};
use fxhash::FxHashMap;
use stdx::hashable_hash_map::{HashableHashMap, HashableHashSet};
use vfs::{AnchoredPath, FileId};

use crate::{Macro, PreprocessingResult, SourcepawnPreprocessor};

#[salsa::query_group(PreprocDatabaseStorage)]
pub trait PreprocDatabase: SourceDatabase {
    #[salsa::invoke(_preprocess_file_query)]
    fn preprocess_file_inner(
        &self,
        file_id: FileId,
        macros: HashableHashMap<String, Macro>,
        being_preprocessed: HashableHashSet<FileId>,
    ) -> Arc<FxHashMap<FileId, Arc<PreprocessingResult>>>;

    #[salsa::invoke(preprocess_file_query)]
    fn preprocess_file(&self, file_id: FileId) -> Arc<PreprocessingResult>;
}

pub(crate) fn preprocess_file_query(
    db: &dyn PreprocDatabase,
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
    db: &dyn PreprocDatabase,
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
