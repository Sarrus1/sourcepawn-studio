use std::sync::Arc;

use lsp_types::{Range, Url};
use parking_lot::RwLock;
use semantic_analyzer::Token;
use syntax::{include_item::IncludeItem, SPItem};

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
    /// * `documents` - Set of known documents.
    /// * `document_uri` - Uri of the document where the include declaration is parsed from.
    pub(crate) fn resolve_import(
        &mut self,
        include_text: &mut String,
        document_uri: &Arc<Url>,
        quoted: bool,
    ) -> Option<Url> {
        // Add the extension to the file if needed.
        let include_text = add_include_extension(include_text, self.environment.amxxpawn_mode);

        if quoted {
            // Search for the relative path.
            let document_path = document_uri.to_file_path().ok()?;
            let parent_path = document_path.parent()?;
            let mut include_file_path = parent_path.join(include_text);
            let mut uri = Url::from_file_path(&include_file_path).ok()?;
            if self.documents.contains_key(&uri) {
                return Some(uri);
            }
            if let Ok(Some(main_path_uri)) = self.environment.options.get_main_path_uri() {
                let main_path = main_path_uri.to_file_path().ok()?;
                let main_path_parent = main_path.parent()?;
                if parent_path != main_path_parent {
                    // Don't look for includes in the include folder if we are not at the root
                    // of the project.
                    return None;
                }
                include_file_path = main_path_parent.join("include").join(include_text);
                log::trace!(
                    "Looking for {:#?} in {:#?}",
                    include_text,
                    include_file_path
                );

                uri = Url::from_file_path(&include_file_path).ok()?;
                if self.documents.contains_key(&uri) {
                    return Some(uri);
                }
                return None;
            }
        }

        // Look for the include in the same directory or the closest include directory.
        let document_path = document_uri.to_file_path().ok()?;
        let document_dirpath = document_path.parent()?;
        let mut include_file_path = document_dirpath.join(include_text);
        log::trace!(
            "Looking for {:#?} in {:#?}",
            include_text,
            include_file_path
        );
        if !include_file_path.exists() {
            log::trace!("{:#?} not found", include_text);
            include_file_path = document_dirpath.join("include").join(include_text);
            log::trace!(
                "Looking for {:#?} in {:#?}",
                include_text,
                include_file_path
            );
        }
        let uri = Url::from_file_path(&include_file_path).ok()?;
        if self.documents.contains_key(&uri) {
            return Some(uri);
        }

        // Look for the includes in the include directories.
        for include_directory in self.environment.options.includes_directories.iter() {
            let path = include_directory.clone().join(include_text);
            let uri = Url::from_file_path(path).ok()?;
            if self.documents.contains_key(&uri) {
                return Some(uri);
            }
        }

        None
    }
}

pub fn add_include(document: &mut Document, include_uri: Url, path: String, range: Range) {
    document.includes.insert(
        include_uri.clone(),
        Token {
            text: path.clone(),
            range,
        },
    );

    let include_uri = Arc::new(include_uri);

    let include_item = IncludeItem {
        name: path,
        range,
        v_range: document.build_v_range(&range),
        uri: document.uri.clone(),
        include_uri,
    };
    let include_item = Arc::new(RwLock::new(SPItem::Include(include_item)));
    document.sp_items.push(include_item);
}
