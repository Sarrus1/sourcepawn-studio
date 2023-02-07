use std::{
    str::Utf8Error,
    sync::{Arc, RwLock},
};

use tree_sitter::Node;

use crate::{
    document::{find_doc, Document, Walker},
    spitem::{
        typedef_item::{Parameter, Type, TypedefItem},
        SPItem,
    },
    utils::ts_range_to_lsp_range,
};

pub fn parse_typedef(
    document: &mut Document,
    node: &Node,
    walker: &mut Walker,
) -> Result<(), Utf8Error> {
    // Name of the typedef
    let name_node = node.child_by_field_name("name");
    // Return type of the typedef
    let mut type_node = None;
    // Parameters of the declaration
    let mut argument_declarations_node = None;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        if kind == "typedef_expression" {
            type_node = child.child_by_field_name("returnType");
            let mut sub_cursor = child.walk();
            for sub_child in child.children(&mut sub_cursor) {
                if sub_child.kind() == "argument_declarations" {
                    argument_declarations_node = Some(sub_child)
                }
            }
        }
    }

    if name_node.is_none() {
        // A typedef always has a name and parameters.
        return Ok(());
    }
    let name_node = name_node.unwrap();
    let name = name_node.utf8_text(document.text.as_bytes());

    let mut type_ = "";
    if let Some(type_node) = type_node {
        type_ = type_node.utf8_text(document.text.as_bytes())?;
    }

    let description = find_doc(walker, node.start_position().row)?;

    let typedef_item = TypedefItem {
        name: name?.to_string(),
        type_: type_.to_string(),
        range: ts_range_to_lsp_range(&name_node.range()),
        full_range: ts_range_to_lsp_range(&node.range()),
        description,
        uri: document.uri.clone(),
        detail: node.utf8_text(document.text.as_bytes())?.to_string(),
        references: vec![],
        params: vec![],
    };

    let typedef_item = Arc::new(RwLock::new(SPItem::Typedef(typedef_item)));
    read_typedef_parameters(
        document,
        argument_declarations_node.unwrap(),
        typedef_item.clone(),
    )?;
    document.sp_items.push(typedef_item);

    Ok(())
}

fn read_typedef_parameters(
    document: &mut Document,
    arguments_declaration_node: Node,
    typedef_item: Arc<RwLock<SPItem>>,
) -> Result<(), Utf8Error> {
    let mut cursor = arguments_declaration_node.walk();
    for child in arguments_declaration_node.children(&mut cursor) {
        match child.kind() {
            "argument_declaration" => {
                let name_node = child.child_by_field_name("name");
                let type_node = child.child_by_field_name("type");
                let mut is_const = false;
                let mut sub_cursor = child.walk();
                for sub_child in child.children(&mut sub_cursor) {
                    let sub_child_text = sub_child.utf8_text(document.text.as_bytes())?;
                    if sub_child_text == "const" {
                        is_const = true;
                    }
                }
                let name_node = name_node.unwrap();
                let name = name_node.utf8_text(document.text.as_bytes());

                let parameter = Parameter {
                    name: name?.to_string(),
                    is_const,
                    type_: parse_argument_type(document, type_node),
                };
                typedef_item
                    .write()
                    .unwrap()
                    .push_type_param(Arc::new(RwLock::new(parameter)));
            }
            "rest_argument" => {
                // TODO: Handle this
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_argument_type(document: &mut Document, argument_type_node: Option<Node>) -> Option<Type> {
    let argument_type_node = argument_type_node?;

    let mut cursor = argument_type_node.walk();
    let mut type_ = Type {
        name: "".to_string(),
        is_pointer: false,
        dimension: vec![],
    };

    for child in argument_type_node.children(&mut cursor) {
        match child.kind() {
            // FIXME: Handle oldtypes.
            "type" => {
                type_.name = child
                    .utf8_text(document.text.as_bytes())
                    .unwrap()
                    .to_string();
            }
            "&" => type_.is_pointer = true,
            "dimension" => {
                type_.dimension.push(
                    child
                        .utf8_text(document.text.as_bytes())
                        .unwrap()
                        .to_string(),
                );
            }
            _ => {}
        }
    }

    Some(type_)
}
