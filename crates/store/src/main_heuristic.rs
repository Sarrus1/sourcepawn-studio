use lsp_types::Url;

use crate::{document::Document, Store};

impl Store {
    /// Check if a document is a potential main file.
    /// Used when the mainPath was not set by the user.
    ///
    /// A document is a potential main file when it is not in an includeDirectory,
    /// if it is a .sp file and it contains `OnPluginStart(`.
    ///
    /// # Arguments
    ///
    /// * `document` - [Document] to check against.
    fn is_main_heuristic(&self, document: &Document) -> Option<Url> {
        if self.environment.amxxpawn_mode {
            return None;
        }
        let path = document.path().ok()?;
        let path = path.to_str()?;
        for include_directory in self.environment.options.includes_directories.iter() {
            if path.contains(include_directory.to_str()?) {
                return None;
            }
        }
        if document.extension().ok()? == "sp" && document.text.contains("OnPluginStart()") {
            return Some(document.uri());
        }

        None
    }

    pub fn find_main_with_heuristic(&self) -> Option<Url> {
        self.documents
            .values()
            .find_map(|document| self.is_main_heuristic(document))
    }
}
