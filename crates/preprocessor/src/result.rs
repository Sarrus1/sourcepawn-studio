use fxhash::FxHashMap;

use crate::{errors::PreprocessorErrors, MacrosMap, Offset};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessingResult {
    preprocessed_text: String,
    macros: MacrosMap,
    offsets: FxHashMap<u32, Vec<Offset>>,
    errors: PreprocessorErrors,
    inactive_ranges: Vec<lsp_types::Range>,
}

impl PreprocessingResult {
    pub(crate) fn new(
        preprocessed_text: String,
        macros: MacrosMap,
        offsets: FxHashMap<u32, Vec<Offset>>,
        errors: PreprocessorErrors,
        inactive_ranges: Vec<lsp_types::Range>,
    ) -> Self {
        Self {
            preprocessed_text,
            macros,
            offsets,
            errors,
            inactive_ranges,
        }
    }

    pub fn default(text: &str) -> Self {
        Self {
            preprocessed_text: text.to_string(),
            macros: FxHashMap::default(),
            offsets: FxHashMap::default(),
            errors: Default::default(),
            inactive_ranges: Default::default(),
        }
    }

    pub fn preprocessed_text(&self) -> &str {
        &self.preprocessed_text
    }

    pub fn macros(&self) -> &MacrosMap {
        &self.macros
    }

    pub fn offsets(&self) -> &FxHashMap<u32, Vec<Offset>> {
        &self.offsets
    }

    pub fn errors(&self) -> &PreprocessorErrors {
        &self.errors
    }

    pub fn inactive_ranges(&self) -> &[lsp_types::Range] {
        &self.inactive_ranges
    }
}
