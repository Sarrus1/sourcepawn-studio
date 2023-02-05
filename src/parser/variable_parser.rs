use std::{
    str::Utf8Error,
    sync::{Arc, RwLock},
};

use tree_sitter::Node;

use crate::{
    document::Document,
    providers::hover::description::Description,
    spitem::{
        variable_item::{VariableItem, VariableStorageClass, VariableVisibility},
        SPItem,
    },
    utils::ts_range_to_lsp_range,
};

pub fn parse_variable(
    file_item: &mut Document,
    node: &mut Node,
    parent: Option<Arc<RwLock<SPItem>>>,
) -> Result<(), Utf8Error> {
    let mut cursor = node.walk();
    // Type of the variable
    let type_node = node.child_by_field_name("type");
    // Visibility of the variable (public, stock)
    let mut visibility: Vec<VariableVisibility> = vec![];
    // Storage class of the variable (public, stock)
    let mut storage_class: Vec<VariableStorageClass> = vec![];

    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind {
            "variable_visibility" => {
                let visibility_text = child.utf8_text(file_item.text.as_bytes())?;
                if visibility_text.contains("stock") {
                    visibility.push(VariableVisibility::Stock);
                }
                if visibility_text.contains("public") {
                    visibility.push(VariableVisibility::Public);
                }
            }
            "variable_storage_class" => {
                let storage_class_text = child.utf8_text(file_item.text.as_bytes())?;
                if storage_class_text.contains("const") {
                    storage_class.push(VariableStorageClass::Const);
                }
                if storage_class_text.contains("static") {
                    storage_class.push(VariableStorageClass::Static);
                }
            }
            "variable_declaration" | "old_variable_declaration" => {
                let name_node = child.child_by_field_name("name").unwrap();
                let mut dimensions: Vec<String> = vec![];

                let mut cursor = child.walk();
                for sub_child in child.children(&mut cursor) {
                    let kind = sub_child.kind();
                    match kind {
                        "fixed_dimension" | "dimension" => {
                            let dimension_text = sub_child.utf8_text(file_item.text.as_bytes())?;
                            dimensions.push(dimension_text.to_string());
                        }
                        _ => {
                            continue;
                        }
                    }
                }
                let type_ = match type_node {
                    Some(type_node) => type_node.utf8_text(file_item.text.as_bytes())?,
                    None => "",
                };
                let name = name_node.utf8_text(file_item.text.as_bytes())?;
                let variable_item = VariableItem {
                    name: name.to_string(),
                    type_: type_.to_string(),
                    range: ts_range_to_lsp_range(&name_node.range()),
                    description: Description::default(),
                    uri: file_item.uri.clone(),
                    detail: "".to_string(),
                    visibility: visibility.clone(),
                    storage_class: storage_class.clone(),
                    parent: parent.clone(),
                    references: vec![],
                };
                let variable_item = Arc::new(RwLock::new(SPItem::Variable(variable_item)));
                file_item.sp_items.push(variable_item);
            }
            _ => {}
        }
    }

    Ok(())
}
