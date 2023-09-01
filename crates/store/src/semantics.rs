use semantic_analyzer::find_references;
use syntax::FileId;

use crate::Store;

impl Store {
    pub fn find_references(&mut self, file_id: &FileId) {
        let uri = self.path_interner.lookup(*file_id);
        log::trace!("Resolving references for document {:?}", uri);
        if !self.documents.contains_key(file_id) {
            log::trace!("Skipped resolving references for document {:?}", uri);
            return;
        }
        let all_items = self.get_all_items(file_id, false);
        let Some(document) = self.documents.get_mut(file_id) else {
            return;
        };
        if let Some(unresolved_tokens) = find_references(
            all_items,
            &document.uri,
            document.file_id,
            &document.preprocessed_text,
            &mut document.tokens,
            &mut document.offsets,
        ) {
            document.unresolved_tokens = unresolved_tokens;
        }
    }
}
