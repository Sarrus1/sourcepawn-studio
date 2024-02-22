use std::sync::Arc;

use fxhash::FxHashMap;

use crate::{errors::PreprocessorErrors, ArgsMap, MacrosMap, Offset};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessingResult {
    preprocessed_text: Arc<str>,
    macros: MacrosMap,
    offsets: FxHashMap<u32, Vec<Offset>>,
    args_map: ArgsMap,
    errors: PreprocessorErrors,
    inactive_ranges: Vec<lsp_types::Range>,
}

impl PreprocessingResult {
    pub(crate) fn new(
        preprocessed_text: Arc<str>,
        macros: MacrosMap,
        offsets: FxHashMap<u32, Vec<Offset>>,
        args_map: ArgsMap,
        errors: PreprocessorErrors,
        inactive_ranges: Vec<lsp_types::Range>,
    ) -> Self {
        Self {
            preprocessed_text,
            macros,
            offsets,
            args_map,
            errors,
            inactive_ranges,
        }
    }

    pub fn sort_offsets(&mut self) {
        for offsets in self.offsets.values_mut() {
            offsets.sort_by(|a, b| match a.range.start.cmp(&b.range.start) {
                std::cmp::Ordering::Equal => a.range.end.cmp(&b.range.end),
                ord => ord,
            });
        }
    }

    pub fn default(text: &str) -> Self {
        Self {
            preprocessed_text: text.to_string().into(),
            macros: FxHashMap::default(),
            offsets: FxHashMap::default(),
            args_map: FxHashMap::default(),
            errors: Default::default(),
            inactive_ranges: Default::default(),
        }
    }

    pub fn preprocessed_text(&self) -> Arc<str> {
        self.preprocessed_text.clone()
    }

    pub fn macros(&self) -> &MacrosMap {
        &self.macros
    }

    pub fn offsets(&self) -> &FxHashMap<u32, Vec<Offset>> {
        &self.offsets
    }

    pub fn args_map(&self) -> &ArgsMap {
        &self.args_map
    }

    pub fn errors(&self) -> &PreprocessorErrors {
        &self.errors
    }

    pub fn inactive_ranges(&self) -> &[lsp_types::Range] {
        &self.inactive_ranges
    }
}
