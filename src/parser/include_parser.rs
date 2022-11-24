use std::{collections::HashMap, str::Utf8Error, sync::Arc};

use lsp_types::Url;
use tree_sitter::Node;

use crate::{document::Document, environment::Environment, utils};

pub fn parse_include(
    environment: &Environment,
    documents: &HashMap<Arc<Url>, Document>,
    document: &mut Document,
    node: &mut Node,
) -> Result<(), Utf8Error> {
    let path_node = node.child_by_field_name("path").unwrap();
    let path = path_node.utf8_text(&document.text.as_bytes())?;
    // Remove leading and trailing "<" and ">" or ".
    let mut path = path[1..path.len() - 1].trim().to_string();
    let uri = resolve_import(environment, &mut path, &documents);
    if uri.is_none() {
        return Ok(());
    }
    let uri = uri.unwrap();
    document.includes.insert(uri);

    Ok(())
}

fn resolve_import(
    environment: &Environment,
    include_text: &mut String,
    documents: &HashMap<Arc<Url>, Document>,
) -> Option<String> {
    let include_directories = &environment.options.includes_directories;
    let include_text = utils::add_include_extension(include_text);
    for include_directory in include_directories.iter() {
        let path = include_directory.clone().join(include_text);
        let uri = Url::from_file_path(path).unwrap();
        if documents.contains_key(&uri) {
            return Some(uri.to_string());
        }
    }

    None
}
