use std::{str::Utf8Error, sync::Arc};

use tree_sitter::Node;

use crate::{
    document::{find_doc, Document, Walker},
    providers::hover::description::Description,
    spitem::{enum_struct_item::EnumStructItem, variable_item::VariableItem, SPItem},
    utils::ts_range_to_lsp_range,
};

use super::function_parser::parse_function;

pub fn parse_enum_struct(
    document: &mut Document,
    node: &mut Node,
    walker: &mut Walker,
) -> Result<(), Utf8Error> {
    // Name of the enum struct
    let name_node = node.child_by_field_name("name").unwrap();
    let name = name_node.utf8_text(&document.text.as_bytes());

    let documentation = find_doc(walker, node.start_position().row, &document.text)?;

    let enum_struct_item = EnumStructItem {
        name: name?.to_string(),
        range: ts_range_to_lsp_range(&name_node.range()),
        full_range: ts_range_to_lsp_range(&node.range()),
        description: documentation,
        uri: document.uri.clone(),
        references: vec![],
    };

    let enum_struct_item = Arc::new(SPItem::EnumStruct(enum_struct_item));
    parse_enum_struct_members(document, node, enum_struct_item.clone(), walker);
    document.sp_items.push(enum_struct_item);

    Ok(())
}

fn parse_enum_struct_members(
    document: &mut Document,
    node: &Node,
    enum_struct_item: Arc<SPItem>,
    walker: &mut Walker,
) {
    let mut cursor = node.walk();
    for mut child in node.children(&mut cursor) {
        match child.kind() {
            "enum_struct_field" => parse_enum_struct_field(document, &child, &enum_struct_item),
            "enum_struct_method" => {
                parse_function(document, &mut child, walker, Some(enum_struct_item.clone()))
                    .unwrap()
            }
            "comment" => { //TODO:
            }
            "preproc_pragma_deprecated" => { //TODO:
            }
            _ => {}
        }
    }
}

fn parse_enum_struct_field(document: &mut Document, node: &Node, enum_struct_item: &Arc<SPItem>) {
    // Name of the enum struct field
    let name_node = node.child_by_field_name("name").unwrap();
    let name = name_node.utf8_text(&document.text.as_bytes()).unwrap();

    let type_node = node.child_by_field_name("type").unwrap();
    let type_ = type_node.utf8_text(&document.text.as_bytes()).unwrap();

    let mut dimensions: Vec<String> = vec![];

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind {
            "fixed_dimension" | "dimension" => {
                let dimension_text = child.utf8_text(&document.text.as_bytes()).unwrap();
                dimensions.push(dimension_text.to_string());
            }
            _ => {
                continue;
            }
        }
    }

    let enum_struct_field_item = VariableItem {
        name: name.to_string(),
        type_: type_.to_string(),
        range: ts_range_to_lsp_range(&name_node.range()),
        description: Description::default(),
        uri: document.uri.clone(),
        detail: format!("{} {}{}", type_, name, dimensions.join("")),
        visibility: vec![],
        storage_class: vec![],
        parent: Some(enum_struct_item.clone()),
        references: vec![],
    };

    let enum_struct_field_item = Arc::new(SPItem::Variable(enum_struct_field_item));

    document.sp_items.push(enum_struct_field_item);
}
