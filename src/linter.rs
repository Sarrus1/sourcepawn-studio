use crate::store::Store;

pub(crate) mod spcomp;

impl Store {
    /// Clear all diagnostics from the documents in the store.
    pub(super) fn clear_all_diagnostics(&mut self) {
        for document in self.documents.values_mut() {
            document.diagnostics.clear();
        }
    }
}
