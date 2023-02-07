use std::{
    str::Utf8Error,
    sync::{Arc, RwLock},
};

use tree_sitter::Node;

use crate::{
    document::{find_doc, Document, Walker},
    providers::hover::description::Description,
    spitem::{
        parameter::{Parameter, Type},
        typedef_item::TypedefItem,
        SPItem,
    },
    utils::ts_range_to_lsp_range,
};

use super::function_parser::extract_param_doc;

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
    document.sp_items.push(typedef_item);

    Ok(())
}

pub(super) fn read_argument_declarations(
    document: &Document,
    argument_declarations_node: Node,
    parent: Arc<RwLock<SPItem>>,
    description: Description,
) -> Result<(), Utf8Error> {
    let mut cursor = argument_declarations_node.walk();
    for child in argument_declarations_node.children(&mut cursor) {
        match child.kind() {
            "argument_declaration" => {
                let name_node = child.child_by_field_name("name");
                let type_node = child.child_by_field_name("type");
                let mut is_const = false;
                let mut dimensions = vec![];
                let mut sub_cursor = child.walk();
                for sub_child in child.children(&mut sub_cursor) {
                    match sub_child.kind() {
                        "const" => is_const = true,
                        "dimension" | "fixed_dimension" => {
                            let dimension = sub_child.utf8_text(document.text.as_bytes())?;
                            dimensions.push(dimension.to_string());
                        }
                        _ => {}
                    }
                }
                let name_node = name_node.unwrap();
                let name = name_node.utf8_text(document.text.as_bytes());

                let parameter = Parameter {
                    name: name?.to_string(),
                    is_const,
                    type_: parse_argument_type(document, type_node),
                    description: Description {
                        text: match extract_param_doc(name?, &description) {
                            Some(text) => text,
                            None => "".to_string(),
                        },
                        deprecated: None,
                    },
                    dimensions,
                };
                parent
                    .write()
                    .unwrap()
                    .push_param(Arc::new(RwLock::new(parameter)));
            }
            "rest_argument" => {
                // TODO: Handle this
            }
            _ => {}
        }
    }
    Ok(())
}

pub(crate) fn parse_argument_type(
    document: &Document,
    argument_type_node: Option<Node>,
) -> Option<Type> {
    let argument_type_node = argument_type_node?;

    let mut cursor = argument_type_node.walk();
    let mut type_ = Type {
        name: "".to_string(),
        is_pointer: false,
        dimensions: vec![],
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
            "dimension" | "fixed_dimension" => {
                type_.dimensions.push(
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
