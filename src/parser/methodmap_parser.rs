use std::{
    str::Utf8Error,
    sync::{Arc, Mutex},
};

use tree_sitter::Node;

use crate::{
    document::{find_doc, Document, Walker},
    spitem::{methodmap_item::MethodmapItem, SPItem},
    utils::ts_range_to_lsp_range,
};

pub fn parse_methodmap(
    document: &mut Document,
    node: &mut Node,
    walker: &mut Walker,
) -> Result<(), Utf8Error> {
    let name_node = node.child_by_field_name("name").unwrap();
    let name = name_node.utf8_text(&document.text.as_bytes()).unwrap();
    let inherit_node = node.child_by_field_name("inherits");
    let inherit = match inherit_node {
        Some(inherit_node) => inherit_node
            .utf8_text(&document.text.as_bytes())
            .unwrap()
            .trim(),
        None => "",
    };

    let methodmap_item = MethodmapItem {
        name: name.to_string(),
        range: ts_range_to_lsp_range(&name_node.range()),
        full_range: ts_range_to_lsp_range(&node.range()),
        parent: None,
        description: find_doc(walker, node.start_position().row)?,
        uri: document.uri.clone(),
        references: vec![],
    };

    let methodmap_item = Arc::new(Mutex::new(SPItem::Methodmap(methodmap_item)));
    document.sp_items.push(methodmap_item);

    Ok(())
}
