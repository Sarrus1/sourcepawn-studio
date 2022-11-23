use std::str::Utf8Error;

use tree_sitter::Node;

use crate::fileitem::Document;

pub(crate) fn parse_include(file_item: &mut Document, node: &mut Node) -> Result<(), Utf8Error> {
    let path_node = node.child_by_field_name("path").unwrap();
    let path = path_node.utf8_text(&file_item.text.as_bytes())?;
    // Remove leading and trailing "<" and ">" or ".
    let path = path[1..path.len() - 1].trim();

    Ok(())
}
