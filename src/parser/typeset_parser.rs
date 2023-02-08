use std::{
    str::Utf8Error,
    sync::{Arc, RwLock},
};

use tree_sitter::Node;

use crate::{
    document::{find_doc, Document, Walker},
    spitem::{typedef_item::TypedefItem, typeset_item::TypesetItem, SPItem},
    utils::ts_range_to_lsp_range,
};

use super::typedef_parser::read_argument_declarations;

pub fn parse_typeset(
    document: &mut Document,
    node: &Node,
    walker: &mut Walker,
) -> Result<(), Utf8Error> {
    // Name of the typeset
    let name_node = node.child_by_field_name("name");
    if name_node.is_none() {
        // A typedef always has a name and parameters.
        return Ok(());
    }
    let name_node = name_node.unwrap();
    let name = name_node.utf8_text(document.text.as_bytes())?;

    let description = find_doc(walker, node.start_position().row)?;

    let mut children = vec![];

    let mut cursor = node.walk();
    let mut counter = -1;
    for child in node.children(&mut cursor) {
        match child.kind() {
            "comment" => walker.push_comment(child, &document.text),
            "preproc_pragma" => walker.push_deprecated(child, &document.text),
            "typedef_expression" => {
                counter += 1;
                let mut argument_declarations_node = None;
                let type_node = child.child_by_field_name("returnType");
                let mut sub_cursor = child.walk();
                for sub_child in child.children(&mut sub_cursor) {
                    if sub_child.kind() == "argument_declarations" {
                        argument_declarations_node = Some(sub_child)
                    }
                }

                let mut type_ = "";
                if let Some(type_node) = type_node {
                    type_ = type_node.utf8_text(document.text.as_bytes())?;
                }

                let description = find_doc(walker, node.start_position().row)?;

                let typedef_item = TypedefItem {
                    name: format!("{}{}", name, counter),
                    type_: type_.to_string(),
                    range: ts_range_to_lsp_range(&name_node.range()),
                    full_range: ts_range_to_lsp_range(&node.range()),
                    description: description.clone(),
                    uri: document.uri.clone(),
                    detail: node.utf8_text(document.text.as_bytes())?.to_string(),
                    references: vec![],
                    params: vec![],
                };

                let typedef_item = Arc::new(RwLock::new(SPItem::Typedef(typedef_item)));
                read_argument_declarations(
                    document,
                    argument_declarations_node.unwrap(),
                    typedef_item.clone(),
                    description,
                )?;
                children.push(typedef_item);
            }
            _ => {}
        }
    }

    let typeset_item = TypesetItem {
        name: name.to_string(),
        range: ts_range_to_lsp_range(&name_node.range()),
        full_range: ts_range_to_lsp_range(&node.range()),
        description,
        uri: document.uri.clone(),
        references: vec![],
        children,
    };

    let typeset_item = Arc::new(RwLock::new(SPItem::Typeset(typeset_item)));
    document.sp_items.push(typeset_item);

    Ok(())
}
