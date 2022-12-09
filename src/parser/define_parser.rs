use std::{
    str::Utf8Error,
    sync::{Arc, Mutex},
};

use tree_sitter::Node;

use crate::{
    document::{find_doc, Document, Walker},
    spitem::{define_item::DefineItem, SPItem},
    utils::ts_range_to_lsp_range,
};

pub fn parse_define(
    document: &mut Document,
    node: &mut Node,
    walker: &mut Walker,
) -> Result<(), Utf8Error> {
    let name_node = node.child_by_field_name("name").unwrap();
    let name = name_node.utf8_text(&document.text.as_bytes()).unwrap();
    let value_node = node.child_by_field_name("value");
    let value = match value_node {
        Some(value_node) => value_node
            .utf8_text(&document.text.as_bytes())
            .unwrap()
            .trim(),
        None => "",
    };

    let define_item = DefineItem {
        name: name.to_string(),
        range: ts_range_to_lsp_range(&name_node.range()),
        full_range: ts_range_to_lsp_range(&node.range()),
        value: value.to_string(),
        description: find_doc(walker, node.start_position().row)?,
        uri: document.uri.clone(),
        references: vec![],
    };

    let define_item = Arc::new(Mutex::new(SPItem::Define(define_item)));
    document.sp_items.push(define_item);

    Ok(())
}
