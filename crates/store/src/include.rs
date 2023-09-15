use std::sync::Arc;

use lsp_types::{Range, Url};
use parking_lot::RwLock;
use semantic_analyzer::Token;
use syntax::{include_item::IncludeItem, FileId, SPItem};

use crate::{document::Document, Store};

/// Add `.inc` to a trimmed include text if it does not have an extension (.sp or .inc).
///
/// # Arguments
///
/// * `include_text` - Include text to edit.
fn add_include_extension(include_text: &mut String, amxxpawn_mode: bool) -> &String {
    if amxxpawn_mode {
        if !include_text.ends_with(".sma") && !include_text.ends_with(".inc") {
            include_text.push_str(".inc");
        }
    } else if !include_text.ends_with(".sp") && !include_text.ends_with(".inc") {
        include_text.push_str(".inc");
    }

    include_text
}

impl Store {
    /// Resolve an include from its `#include` directive and the file it was imported in.
    ///
    /// # Arguments
    ///
    /// * `include_directories` - List of directories to look for includes files.
    /// * `include_text` - Text of the include such as `"file.sp"` or `<file>`.
    /// * `document_uri` - Uri of the document where the include declaration is parsed from.
    pub fn resolve_import(
        &self,
        include_text: &mut String,
        document_uri: &Arc<Url>,
        quoted: bool,
    ) -> Option<FileId> {
        // Add the extension to the file if needed.
        let include_text = add_include_extension(include_text, self.environment.amxxpawn_mode);

        if quoted {
            // Search for the relative path.
            let document_path = document_uri.to_file_path().ok()?;
            let parent_path = document_path.parent()?;
            let include_file_path = parent_path.join(include_text);
            let uri = Url::from_file_path(&include_file_path).ok()?;
            if self.contains_uri(&uri) {
                return self.path_interner.get(&uri);
            }
        }

        // Walk backwards in the parents directory to find the include.
        // Look both in the parent and in a directory called `include`.
        // Limit the search to 5 levels.
        // This approach fixes the egg and chicken issue where the main file has to be known in
        // order to resolve the includes, and the includes have to be resolved in order to know
        // the main file.
        let mut document_path = document_uri.to_file_path().ok()?;
        let mut include_file_path = document_path.parent()?.join(include_text);
        let mut i = 0u8;
        while let Some(parent) = document_path.parent().map(|p| p.to_path_buf()) {
            document_path = parent.clone();
            if i > 5 {
                return None;
            }
            i += 1;
            include_file_path = parent.join(include_text);
            log::trace!(
                "Looking for {:#?} in {:#?}",
                include_text,
                include_file_path
            );
            if include_file_path.exists() {
                break;
            }
            log::trace!("{:#?} not found", include_text);
            include_file_path = parent.join("include").join(include_text);
            log::trace!(
                "Looking for {:#?} in {:#?}",
                include_text,
                include_file_path
            );
            if include_file_path.exists() {
                break;
            }
        }
        let uri = Url::from_file_path(&include_file_path).ok()?;
        if self.contains_uri(&uri) {
            return self.path_interner.get(&uri);
        }

        // Look for the includes in the include directories.
        for include_directory in self.environment.options.includes_directories.iter() {
            let path = include_directory.clone().join(include_text);
            let uri = Url::from_file_path(path).ok()?;
            if self.contains_uri(&uri) {
                return self.path_interner.get(&uri);
            }
        }

        None
    }

    pub fn add_include(
        &self,
        document: &mut Document,
        include_id: FileId,
        path: String,
        range: Range,
    ) {
        let include_uri = Arc::new(self.path_interner.lookup(include_id).clone());
        document.includes.insert(
            include_id,
            Token {
                text: path.clone(),
                range,
            },
        );

        let include_item = IncludeItem {
            name: path,
            range,
            v_range: document.build_v_range(&range),
            uri: document.uri.clone(),
            file_id: document.file_id,
            include_uri,
            include_id,
        };
        let include_item = Arc::new(RwLock::new(SPItem::Include(include_item)));
        document.sp_items.push(include_item);
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;
    use tempfile::tempdir;

    fn add_file(store: &mut Store, path: &PathBuf, text: &str) -> Arc<Url> {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
        let uri = Arc::new(Url::from_file_path(path).unwrap());
        let file_id = store.path_interner.intern(uri.as_ref().clone());
        store.documents.insert(
            file_id,
            Document::new(uri.clone(), file_id, text.to_string()),
        );
        uri
    }

    #[test]
    fn test_resolve_include() {
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_owned();

        let mut store = Store::default();

        let path_0 = temp_dir_path.join("main.sp");
        let uri_0 = add_file(&mut store, &path_0, "");

        let path_1 = temp_dir_path.join("include/a.sp");
        let uri_1 = add_file(&mut store, &path_1, "");

        let path_2 = temp_dir_path.join("include/b.inc");
        let uri_2 = add_file(&mut store, &path_2, "");

        let path_3 = temp_dir_path.join("include/others/c.inc");
        let uri_3 = add_file(&mut store, &path_3, "");

        // from main.sp:
        // #include <third>
        assert_eq!(
            store.resolve_import(&mut "b".to_string(), &uri_0, false),
            store.path_interner.get(&uri_2)
        );

        // from main.sp:
        // #include "third"
        assert_eq!(
            store.resolve_import(&mut "b".to_string(), &uri_0, true),
            store.path_interner.get(&uri_2)
        );

        // from main.sp:
        // #include <a.sp>
        assert_eq!(
            store.resolve_import(&mut "a.sp".to_string(), &uri_0, false),
            store.path_interner.get(&uri_1)
        );

        // from main.sp:
        // #include "a.sp"
        assert_eq!(
            store.resolve_import(&mut "a.sp".to_string(), &uri_0, true),
            store.path_interner.get(&uri_1)
        );

        // from a.sp:
        // #include <b>
        assert_eq!(
            store.resolve_import(&mut "b".to_string(), &uri_1, false),
            store.path_interner.get(&uri_2)
        );

        // from c.sp:
        // #include <b>
        assert_eq!(
            store.resolve_import(&mut "b".to_string(), &uri_3, false),
            store.path_interner.get(&uri_2)
        );
    }

    #[test]
    fn test_add_include_extension() {
        let mut include_text = String::from("file");
        add_include_extension(&mut include_text, false);
        assert_eq!(include_text, "file.inc");

        let mut include_text = String::from("file.inc");
        add_include_extension(&mut include_text, false);
        assert_eq!(include_text, "file.inc");

        let mut include_text = String::from("file.sp");
        add_include_extension(&mut include_text, false);
        assert_eq!(include_text, "file.sp");

        let mut include_text = String::from("file");
        add_include_extension(&mut include_text, true);
        assert_eq!(include_text, "file.inc");

        let mut include_text = String::from("file.inc");
        add_include_extension(&mut include_text, true);
        assert_eq!(include_text, "file.inc");

        let mut include_text = String::from("file.sma");
        add_include_extension(&mut include_text, true);
        assert_eq!(include_text, "file.sma");
    }
}
