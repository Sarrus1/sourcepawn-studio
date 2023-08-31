use lsp_types::Url;
use semantic_analyzer::find_references;

use crate::Store;

impl Store {
    pub fn find_references(&mut self, uri: &Url) {
        log::trace!("Resolving references for document {:?}", uri);
        if !self.documents.contains_key(uri) {
            log::trace!("Skipped resolving references for document {:?}", uri);
            return;
        }
        let all_items = self.get_all_items(uri, false);
        let Some(document) = self.documents.get_mut(uri) else{return;};

        if let Some(unresolved_tokens) = find_references(
            all_items,
            &document.uri,
            &document.preprocessed_text,
            &mut document.tokens,
            &mut document.offsets,
        ) {
            document.unresolved_tokens = unresolved_tokens;
        }
    }
}
