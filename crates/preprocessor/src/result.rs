use std::sync::Arc;

use fxhash::FxHashMap;
use sourcepawn_lexer::TextRange;

use crate::{errors::PreprocessorErrors, macros::MacrosMap, offset::SourceMap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessingResult {
    preprocessed_text: Arc<str>,
    macros: MacrosMap,
    source_map: SourceMap,
    errors: PreprocessorErrors,
    inactive_ranges: Vec<TextRange>,
}

impl PreprocessingResult {
    pub(crate) fn new(
        preprocessed_text: Arc<str>,
        macros: MacrosMap,
        mut source_map: SourceMap,
        errors: PreprocessorErrors,
        inactive_ranges: Vec<TextRange>,
    ) -> Self {
        source_map.sort();
        Self {
            preprocessed_text,
            macros,
            source_map,
            errors,
            inactive_ranges,
        }
    }

    pub fn shrink_to_fit(&mut self) {
        self.macros.shrink_to_fit();
        self.source_map.shrink_to_fit();
        self.errors.shrink_to_fit();
        self.inactive_ranges.shrink_to_fit();
    }

    pub fn default(text: &str) -> Self {
        let mut source_map = SourceMap::default();
        source_map.set_preprocecessed_text_len(text.len());
        source_map.set_source_len(text.len());
        Self {
            preprocessed_text: text.to_string().into(),
            macros: FxHashMap::default(),
            source_map,
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

    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    pub fn errors(&self) -> &PreprocessorErrors {
        &self.errors
    }

    pub fn inactive_ranges(&self) -> &[TextRange] {
        &self.inactive_ranges
    }
}
