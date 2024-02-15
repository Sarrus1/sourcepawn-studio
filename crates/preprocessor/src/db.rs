use std::sync::Arc;

use base_db::{infer_include_ext, SourceDatabase};
use fxhash::FxHashMap;
use stdx::hashable_hash_map::{HashableHashMap, HashableHashSet};
use vfs::{AnchoredPath, FileId};

use crate::{Macro, PreprocessingResult, SourcepawnPreprocessor};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PreprocessingParams {
    input_macros: HashableHashMap<String, Macro>,
    output_macros: HashableHashMap<FileId, HashableHashMap<String, Macro>>,
    being_preprocessed: HashableHashSet<FileId>,
}

#[salsa::query_group(PreprocDatabaseStorage)]
pub trait PreprocDatabase: SourceDatabase {
    #[salsa::invoke(_preprocess_file_params_query)]
    fn preprocess_file_inner_params(
        &self,
        file_id: FileId,
        macros: HashableHashMap<String, Macro>,
        being_preprocessed: HashableHashSet<FileId>,
    ) -> Arc<FxHashMap<FileId, Arc<PreprocessingParams>>>;

    #[salsa::invoke(_preprocess_file_data_query)]
    fn preprocess_file_inner_data(
        &self,
        file_id: FileId,
        params: Arc<PreprocessingParams>,
    ) -> Arc<PreprocessingResult>;

    #[salsa::invoke(preprocess_file_query)]
    fn preprocess_file(&self, file_id: FileId) -> Arc<PreprocessingResult>;
}

pub(crate) fn preprocess_file_query(
    db: &dyn PreprocDatabase,
    file_id: FileId,
) -> Arc<PreprocessingResult> {
    // FIXME: Handle errors
    let root_file_id = db.projet_subgraph(file_id).unwrap().root.file_id;
    let res = db.preprocess_file_inner_params(
        root_file_id,
        HashableHashMap::default(),
        HashableHashSet::default(),
    );
    let params = res.get(&root_file_id).unwrap().clone();

    db.preprocess_file_inner_data(file_id, params)
}

pub(crate) fn _preprocess_file_params_query(
    db: &dyn PreprocDatabase,
    file_id: FileId,
    macros: HashableHashMap<String, Macro>,
    mut being_preprocessed: HashableHashSet<FileId>,
) -> Arc<FxHashMap<FileId, Arc<PreprocessingParams>>> {
    being_preprocessed.set.insert(file_id);
    let text = db.file_text(file_id);
    let mut results: FxHashMap<FileId, Arc<PreprocessingParams>> = FxHashMap::default();
    let input_macros = macros.clone();
    let being_preprocessed = being_preprocessed.clone();
    let mut output_macros: HashableHashMap<FileId, HashableHashMap<String, Macro>> =
        HashableHashMap::default();

    let mut preprocessor = SourcepawnPreprocessor::new(file_id, &text);
    preprocessor.set_macros(macros.to_map().clone());
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
                let map = macros.clone().into();
                let res =
                    db.preprocess_file_inner_params(inc_file_id, map, being_preprocessed.clone());
                results.extend(res.as_ref().clone()); //FIXME: Maybe find a way to avoid this clone?
                output_macros.map.extend(
                    res.as_ref()
                        .get(&inc_file_id)
                        .unwrap()
                        .output_macros
                        .map
                        .clone(),
                );
                macros.extend(
                    res.as_ref()
                        .get(&inc_file_id)
                        .unwrap()
                        .output_macros
                        .get(&inc_file_id)
                        .unwrap()
                        .map
                        .clone(),
                );

                Ok(())
            }),
        )
        .unwrap()
        .result();

    output_macros.insert(file_id, res.macros().clone().into());
    results.insert(
        file_id,
        Arc::new(PreprocessingParams {
            input_macros,
            output_macros,
            being_preprocessed,
        }),
    );

    results.into()
}

pub(crate) fn _preprocess_file_data_query(
    db: &dyn PreprocDatabase,
    file_id: FileId,
    params: Arc<PreprocessingParams>,
) -> Arc<PreprocessingResult> {
    let text = db.file_text(file_id);
    let mut preprocessor = SourcepawnPreprocessor::new(file_id, &text);
    preprocessor.set_macros(params.input_macros.map.clone());
    preprocessor
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
                macros.extend(
                    params
                        .as_ref()
                        .output_macros
                        .get(&inc_file_id)
                        .unwrap()
                        .map
                        .clone(),
                );

                Ok(())
            }),
        )
        .unwrap()
        .result()
        .into()
}
