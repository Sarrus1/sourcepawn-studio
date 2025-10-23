use std::sync::Arc;

use anyhow::bail;
use base_db::{infer_include_ext, SourceDatabase};
use fxhash::FxHashMap;
use stdx::hashable_hash_map::{HashableHashMap, HashableHashSet};
use vfs::{AnchoredPath, FileId};

use crate::{HMacrosMap, MacrosMap, PreprocessingResult, SourcepawnPreprocessor};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PreprocessingParams {
    input_macros: HMacrosMap,
    output_macros: HashableHashMap<FileId, HMacrosMap>,
    being_preprocessed: HashableHashSet<FileId>,
}

impl PreprocessingParams {
    pub fn shrink_to_fit(&mut self) {
        self.input_macros.shrink_to_fit();
        self.output_macros.shrink_to_fit();
        self.being_preprocessed.shrink_to_fit();
    }
}

#[salsa::query_group(PreprocDatabaseStorage)]
pub trait PreprocDatabase: SourceDatabase {
    #[salsa::invoke(_preprocess_file_params_query)]
    fn preprocess_file_inner_params(
        &self,
        file_id: FileId,
        macros: HMacrosMap,
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

    #[salsa::invoke(preprocessed_text_query)]
    fn preprocessed_text(&self, file_id: FileId) -> Arc<str>;
}

pub(crate) fn preprocess_file_query(
    db: &dyn PreprocDatabase,
    file_id: FileId,
) -> Arc<PreprocessingResult> {
    let Some(subgraph) = db.projet_subgraph(file_id) else {
        log::warn!("No subgraph found for file_id: {}", file_id);
        return Arc::new(PreprocessingResult::default(db.file_text(file_id).as_ref()));
    };
    let root_file_id = subgraph.root.file_id;
    let res = db.preprocess_file_inner_params(
        root_file_id,
        HashableHashMap::default(),
        HashableHashSet::default(),
    );
    let Some(params) = res.get(&file_id) else {
        log::warn!("No preprocessing params found for file_id: {}", file_id);
        return Arc::new(PreprocessingResult::default(db.file_text(file_id).as_ref()));
    };

    db.preprocess_file_inner_data(file_id, params.clone())
}

pub(crate) fn preprocessed_text_query(db: &dyn PreprocDatabase, file_id: FileId) -> Arc<str> {
    let res = db.preprocess_file(file_id);

    res.preprocessed_text()
}

pub(crate) fn _preprocess_file_params_query(
    db: &dyn PreprocDatabase,
    file_id: FileId,
    macros: HMacrosMap,
    mut being_preprocessed: HashableHashSet<FileId>,
) -> Arc<FxHashMap<FileId, Arc<PreprocessingParams>>> {
    being_preprocessed.insert(file_id);
    let text = db.file_text(file_id);
    let mut results: FxHashMap<FileId, Arc<PreprocessingParams>> = FxHashMap::default();
    let input_macros = macros.clone();
    let mut being_preprocessed = being_preprocessed.clone();
    let mut output_macros: HashableHashMap<FileId, HMacrosMap> = HashableHashMap::default();

    let mut extend_macros =
        |macros: &mut MacrosMap, mut path: String, file_id: FileId, quoted: bool| {
            let mut inc_file_id = None;
            infer_include_ext(&mut path);
            if quoted {
                inc_file_id = db.resolve_path(AnchoredPath::new(file_id, &path));
                if inc_file_id.is_none() {
                    // Hack to try and resolve files in include folder.
                    let path_with_include = format!("include/{}", path);
                    inc_file_id = db.resolve_path(AnchoredPath::new(file_id, &path_with_include));
                }
            };
            if inc_file_id.is_none() {
                inc_file_id = db.resolve_path_relative_to_roots(&path);
            }
            if inc_file_id.is_none() {
                inc_file_id = db.resolve_path(AnchoredPath::new(file_id, &path));
            }
            if inc_file_id.is_none() && !quoted {
                let path_with_include = format!("include/{}", path);
                inc_file_id = db
                    .resolve_path_relative_to_roots(&path_with_include)
                    .or_else(|| db.resolve_path(AnchoredPath::new(file_id, &path_with_include)));
            }
            let inc_file_id = inc_file_id.ok_or_else(|| anyhow::anyhow!("Include not found"))?;
            if being_preprocessed.contains(&inc_file_id) {
                // Avoid cyclic deps
                return Ok(());
            }
            let res = db.preprocess_file_inner_params(
                inc_file_id,
                macros.clone().into(),
                being_preprocessed.clone(),
            );
            results.extend(res.as_ref().clone());
            being_preprocessed.extend(res[&inc_file_id].being_preprocessed.clone());

            let Some(params) = res.as_ref().get(&inc_file_id) else {
                bail!("No preprocessing params found for file_id: {}", inc_file_id);
            };
            output_macros.extend(params.output_macros.clone());
            macros.extend(
                params
                    .output_macros
                    .get(&inc_file_id)
                    .map(|m| m.to_map().clone())
                    .unwrap_or_default(),
            );

            Ok(())
        };

    let mut preprocessor = SourcepawnPreprocessor::new(file_id, &text, &mut extend_macros);
    preprocessor.set_macros(macros.to_map());
    let res = preprocessor.preprocess_input();

    output_macros.insert(file_id, res.macros().clone().into());
    let mut preprocessing_params = PreprocessingParams {
        input_macros,
        output_macros,
        being_preprocessed,
    };
    preprocessing_params.shrink_to_fit();
    results.insert(file_id, preprocessing_params.into());
    results.shrink_to_fit();

    results.into()
}

pub(crate) fn _preprocess_file_data_query(
    db: &dyn PreprocDatabase,
    file_id: FileId,
    params: Arc<PreprocessingParams>,
) -> Arc<PreprocessingResult> {
    let text = db.file_text(file_id);
    let mut extend_macros =
        |macros: &mut MacrosMap, mut path: String, file_id: FileId, quoted: bool| {
            let mut inc_file_id = None;
            infer_include_ext(&mut path);
            if quoted {
                inc_file_id = db.resolve_path(AnchoredPath::new(file_id, &path));
                // FIXME: Investigate why uncommenting this causes upstream macros to not be found.
                // if inc_file_id.is_none() {
                //     // Hack to try and resolve files in include folder.
                //     let path_with_include = format!("include/{}", path);
                //     inc_file_id = db.resolve_path(AnchoredPath::new(file_id, &path_with_include));
                // }
            };
            if inc_file_id.is_none() {
                inc_file_id = db.resolve_path_relative_to_roots(&path);
            }
            if inc_file_id.is_none() {
                inc_file_id = db.resolve_path(AnchoredPath::new(file_id, &path));
            }
            if inc_file_id.is_none() && !quoted {
                let path_with_include = format!("include/{}", path);
                inc_file_id = db
                    .resolve_path_relative_to_roots(&path_with_include)
                    .or_else(|| db.resolve_path(AnchoredPath::new(file_id, &path_with_include)));
            }
            let inc_file_id = inc_file_id.ok_or_else(|| anyhow::anyhow!("Include not found"))?;
            macros.extend(
                params
                    .as_ref()
                    .output_macros
                    .get(&inc_file_id)
                    .map(|m| m.to_map().clone())
                    .unwrap_or_default(),
            );

            Ok(())
        };

    let mut preprocessor = SourcepawnPreprocessor::new(file_id, &text, &mut extend_macros);
    preprocessor.set_macros(params.input_macros.to_map());

    preprocessor.preprocess_input().into()
}
