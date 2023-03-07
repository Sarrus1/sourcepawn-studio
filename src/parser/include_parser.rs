use std::{
    collections::HashMap,
    path::PathBuf,
    str::Utf8Error,
    sync::{Arc, RwLock},
};

use lsp_types::Url;
use tree_sitter::Node;

use crate::{
    document::Document,
    spitem::{include_item::IncludeItem, SPItem},
    store::Store,
    utils::{self, ts_range_to_lsp_range},
};

impl Store {
    pub(crate) fn parse_include(
        &mut self,
        document: &mut Document,
        node: &mut Node,
    ) -> Result<(), Utf8Error> {
        let path_node = node.child_by_field_name("path").unwrap();
        let path = path_node.utf8_text(document.text.as_bytes())?;

        // Remove leading and trailing "<" and ">" or ".
        if path.len() < 2 {
            // The include path is empty.
            return Ok(());
        }
        let mut path = path[1..path.len() - 1].trim().to_string();
        let include_uri = resolve_import(
            &self.environment.options.includes_directories,
            &mut path,
            &self.documents,
            &document.uri,
        );
        if include_uri.is_none() {
            // The include was not found.
            document.missing_includes.insert(path);
            return Ok(());
        }

        let include_uri = include_uri.unwrap();
        document.includes.insert(include_uri.clone());

        let include_uri = Arc::new(include_uri);

        let include_item = IncludeItem {
            name: path,
            range: ts_range_to_lsp_range(&path_node.range()),
            uri: document.uri.clone(),
            include_uri,
        };
        let include_item = Arc::new(RwLock::new(SPItem::Include(include_item)));
        document.sp_items.push(include_item);

        Ok(())
    }
}

/// Resolve an include from its `#include` directive and the file it was imported in.
///
/// # Arguments
///
/// * `include_directories` - List of directories to look for includes files.
/// * `include_text` - Text of the include such as `"file.sp"` or `<file>`.
/// * `documents` - List of known documents.
/// * `document_uri` - Uri of the document where the include declaration is parsed from.
fn resolve_import(
    include_directories: &[PathBuf],
    include_text: &mut String,
    documents: &HashMap<Arc<Url>, Document>,
    document_uri: &Arc<Url>,
) -> Option<Url> {
    // Add the extension to the file if needed.
    let include_text = utils::add_include_extension(include_text);

    // Look for the include in the same directory or the closest include directory.
    let document_path = document_uri.to_file_path().unwrap();
    let document_dirpath = document_path.parent().unwrap();
    let mut include_file_path = document_dirpath.join(include_text);
    if !include_file_path.exists() {
        include_file_path = document_dirpath.join("include").join(include_text);
    }
    let uri = Url::from_file_path(&include_file_path).unwrap();
    if documents.contains_key(&uri) {
        return Some(uri);
    }

    // Look for the includes in the include directories.
    for include_directory in include_directories.iter() {
        let path = include_directory.clone().join(include_text);
        let uri = Url::from_file_path(path).unwrap();
        if documents.contains_key(&uri) {
            return Some(uri);
        }
    }

    None
}
