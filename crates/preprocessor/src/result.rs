use fxhash::FxHashMap;

use crate::{errors::EvaluationError, Macro, Offset};

#[derive(Debug, Clone, Eq)]
pub struct PreprocessingResult {
    preprocessed_text: String,
    macros: FxHashMap<String, Macro>,
    offsets: FxHashMap<u32, Vec<Offset>>,
    evaluation_errors: Vec<EvaluationError>,
}

impl PartialEq for PreprocessingResult {
    fn eq(&self, other: &Self) -> bool {
        // HACK: This allows salsa to only care about the macros map when comparing between
        // different results. The preprocessed text and offsets will almost always be different
        // but the macros map is the only thing that salsa cares about (hopefully).
        // This might cause issues in the future ?
        self.macros == other.macros
    }
}

impl PreprocessingResult {
    pub(crate) fn new(
        preprocessed_text: String,
        macros: FxHashMap<String, Macro>,
        offsets: FxHashMap<u32, Vec<Offset>>,
        evaluation_errors: Vec<EvaluationError>,
    ) -> Self {
        Self {
            preprocessed_text,
            macros,
            offsets,
            evaluation_errors,
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

    pub fn evaluation_errors(&self) -> &Vec<EvaluationError> {
        &self.evaluation_errors
    }
}
