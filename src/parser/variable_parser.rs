use std::str::Utf8Error;

use tree_sitter::Node;

use crate::{
    document::Document,
    spitem::{
        variable_item::{VariableItem, VariableVisibility},
        SPItem,
    },
    utils::ts_range_to_lsp_range,
};

pub fn parse_variable(file_item: &mut Document, node: &mut Node) -> Result<(), Utf8Error> {
    let mut cursor = node.walk();
    // Type of the variable
    let type_node = node.child_by_field_name("type");
    // Visibility of the variable (public, static, stock)
    let mut visibility: Vec<VariableVisibility> = vec![];
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind {
            "variable_storage_class" => {
                let visibility_text = child.utf8_text(&file_item.text.as_bytes())?;
                if visibility_text.contains("stock") {
                    visibility.push(VariableVisibility::Stock);
                }
                if visibility_text.contains("public") {
                    visibility.push(VariableVisibility::Public);
                }
                if visibility_text.contains("static") {
                    visibility.push(VariableVisibility::Static);
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
                            let dimension_text = sub_child.utf8_text(&file_item.text.as_bytes())?;
                            dimensions.push(dimension_text.to_string());
                        }
                        _ => {
                            continue;
                        }
                    }
                }
                let type_ = match type_node {
                    Some(type_node) => type_node.utf8_text(&file_item.text.as_bytes())?,
                    None => "",
                };
                let name = name_node.utf8_text(&file_item.text.as_bytes())?;
                let variable_item = VariableItem {
                    name: name.to_string(),
                    type_: type_.to_string(),
                    range: ts_range_to_lsp_range(&name_node.range()),
                    description: "".to_string(),
                    uri: file_item.uri.clone(),
                    deprecated: false,
                    detail: "".to_string(),
                    visibility: visibility.clone(),
                };
                file_item.sp_items.push(SPItem::Variable(variable_item));
            }
            _ => {}
        }
    }

    Ok(())
}
