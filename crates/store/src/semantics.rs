use path_interner::FileId;
use semantic_analyzer::resolve_references;

use crate::Store;

impl Store {
    /// Resolve all references in a document given by its [file_id](FileId).
    ///
    /// # Arguments
    /// * `file_id` - The [file_id](FileId) of the document to resolve.
    pub fn resolve_file_references(&mut self, file_id: &FileId) {
        log::trace!(
            "Resolving references for file {:?}",
            self.path_interner.lookup(*file_id)
        );
        if !self.documents.contains_key(file_id) {
            log::trace!(
                "Skipped resolving references for document {:?}",
                self.path_interner.lookup(*file_id)
            );
            return;
        }
        let all_items = self.get_all_items(file_id, false);
        let Some(document) = self.documents.get_mut(file_id) else {
            return;
        };
        if let Some(unresolved_tokens) = resolve_references(
            all_items,
            &document.uri,
            document.file_id,
            &document.preprocessed_text,
            &mut document.tokens,
            &mut document.offsets,
        ) {
            document.unresolved_tokens = unresolved_tokens;
        }
        document.mark_as_resolved();
        log::trace!(
            "Done resolving references for file {:?}",
            self.path_interner.lookup(*file_id)
        );
    }
}
