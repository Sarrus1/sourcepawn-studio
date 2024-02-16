use fxhash::FxHashMap;

use crate::{errors::PreprocessorErrors, Macro, Offset};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessingResult {
    preprocessed_text: String,
    macros: FxHashMap<String, Macro>,
    offsets: FxHashMap<u32, Vec<Offset>>,
    errors: PreprocessorErrors,
}

impl PreprocessingResult {
    pub(crate) fn new(
        preprocessed_text: String,
        macros: FxHashMap<String, Macro>,
        offsets: FxHashMap<u32, Vec<Offset>>,
        errors: PreprocessorErrors,
    ) -> Self {
        Self {
            preprocessed_text,
            macros,
            offsets,
            errors,
        }
    }

    pub fn default(text: &str) -> Self {
        Self {
            preprocessed_text: text.to_string(),
            macros: FxHashMap::default(),
            offsets: FxHashMap::default(),
            errors: Default::default(),
        }
    }

    pub fn preprocessed_text(&self) -> &str {
        &self.preprocessed_text
    }

    pub fn macros(&self) -> &FxHashMap<String, Macro> {
        &self.macros
    }

    pub fn offsets(&self) -> &FxHashMap<u32, Vec<Offset>> {
        &self.offsets
    }

    pub fn errors(&self) -> &PreprocessorErrors {
        &self.errors
    }
}
