use fxhash::FxHashMap;

use crate::{errors::EvaluationError, Macro, Offset};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessingResult {
    preprocessed_text: String,
    macros: FxHashMap<String, Macro>,
    offsets: FxHashMap<u32, Vec<Offset>>,
    evaluation_errors: Vec<EvaluationError>,
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

    pub fn default(text: &str) -> Self {
        Self {
            preprocessed_text: text.to_string(),
            macros: FxHashMap::default(),
            offsets: FxHashMap::default(),
            evaluation_errors: Vec::new(),
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
