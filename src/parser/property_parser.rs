use std::{
    str::Utf8Error,
    sync::{Arc, RwLock},
};

use tree_sitter::Node;

use crate::{
    document::{find_doc, Document, Walker},
    spitem::{property_item::PropertyItem, SPItem},
    utils::ts_range_to_lsp_range,
};

pub fn parse_property(
    document: &mut Document,
    node: &mut Node,
    walker: &mut Walker,
    parent: Arc<RwLock<SPItem>>,
) -> Result<(), Utf8Error> {
    let name_node = node.child_by_field_name("name").unwrap();
    let name = name_node.utf8_text(document.text.as_bytes()).unwrap();
    let type_node = node.child_by_field_name("type").unwrap();
    let type_ = type_node.utf8_text(document.text.as_bytes()).unwrap();

    let property_item = PropertyItem {
        name: name.to_string(),
        range: ts_range_to_lsp_range(&name_node.range()),
        full_range: ts_range_to_lsp_range(&node.range()),
        type_: type_.to_string(),
        description: find_doc(walker, node.start_position().row)?,
        uri: document.uri.clone(),
        references: vec![],
        parent: Arc::downgrade(&parent),
    };

    let property_item = Arc::new(RwLock::new(SPItem::Property(property_item)));
    document.sp_items.push(property_item);
    // TODO: Add getter and setter parsing.
    Ok(())
}
