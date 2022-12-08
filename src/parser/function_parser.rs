use std::{str::Utf8Error, sync::Arc};

use tree_sitter::{Node, QueryCursor, QueryMatch};

use crate::{
    document::{find_doc, Document, Walker},
    providers::hover::description::Description,
    spitem::{
        function_item::{FunctionDefinitionType, FunctionItem, FunctionVisibility},
        variable_item::{VariableItem, VariableStorageClass},
        SPItem,
    },
    utils::ts_range_to_lsp_range,
};

use super::{variable_parser::parse_variable, VARIABLE_QUERY};

pub fn parse_function(
    file_item: &mut Document,
    node: &Node,
    walker: &mut Walker,
    parent: Option<Arc<SPItem>>,
) -> Result<(), Utf8Error> {
    // Name of the function
    let name_node = node.child_by_field_name("name");
    // Return type of the function
    let type_node = node.child_by_field_name("returnType");
    // Visibility of the function (public, static, stock)
    let mut visibility_node: Option<Node> = None;
    // Parameters of the declaration
    let mut params_node: Option<Node> = None;
    // Type of function definition ("native" or "forward")
    let mut definition_type_node: Option<Node> = None;

    let mut block_node: Option<Node> = None;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        match kind {
            "function_visibility" => {
                visibility_node = Some(child);
            }
            "argument_declarations" => {
                params_node = Some(child);
            }
            "function_definition_type" => {
                definition_type_node = Some(child);
            }
            "block" => {
                block_node = Some(child);
            }
            _ => {
                continue;
            }
        }
    }

    if name_node.is_none() {
        // A function always has a name.
        return Ok(());
    }
    let name_node = name_node.unwrap();
    let name = name_node.utf8_text(&file_item.text.as_bytes());

    let mut type_ = Ok("");
    if type_node.is_some() {
        type_ = type_node.unwrap().utf8_text(&file_item.text.as_bytes());
    }

    let mut visibility = vec![];
    if visibility_node.is_some() {
        let visibility_text = visibility_node
            .unwrap()
            .utf8_text(&file_item.text.as_bytes())?;
        if visibility_text.contains("stock") {
            visibility.push(FunctionVisibility::Stock);
        }
        if visibility_text.contains("public") {
            visibility.push(FunctionVisibility::Public);
        }
        if visibility_text.contains("static") {
            visibility.push(FunctionVisibility::Static);
        }
    }

    let mut definition_type = FunctionDefinitionType::None;
    if definition_type_node.is_some() {
        definition_type = match definition_type_node
            .unwrap()
            .utf8_text(&file_item.text.as_bytes())?
        {
            "forward" => FunctionDefinitionType::Forward,
            "native" => FunctionDefinitionType::Native,
            _ => FunctionDefinitionType::None,
        }
    }
    let documentation = find_doc(walker, node.start_position().row, &file_item.text)?;

    let function_item = FunctionItem {
        name: name?.to_string(),
        type_: type_?.to_string(),
        range: ts_range_to_lsp_range(&name_node.range()),
        full_range: ts_range_to_lsp_range(&node.range()),
        description: documentation,
        uri: file_item.uri.clone(),
        detail: "".to_string(),
        visibility,
        definition_type,
        references: vec![],
        parent,
    };
    let function_item = Arc::new(SPItem::Function(function_item));
    match block_node {
        Some(block_node) => read_body_variables(
            file_item,
            block_node,
            file_item.text.to_string(),
            function_item.clone(),
        )?,
        None => {}
    }
    read_function_parameters(
        file_item,
        params_node,
        file_item.text.to_string(),
        function_item.clone(),
    )?;
    file_item.sp_items.push(function_item);

    Ok(())
}

fn read_body_variables(
    file_item: &mut Document,
    block_node: Node,
    text: String,
    function_item: Arc<SPItem>,
) -> Result<(), Utf8Error> {
    let mut cursor = QueryCursor::new();
    let matches = cursor
        .matches(&VARIABLE_QUERY, block_node, text.as_bytes())
        .collect::<Vec<QueryMatch>>();
    for match_ in matches.iter() {
        for capture in match_.captures.iter() {
            parse_variable(
                file_item,
                &mut capture.node.clone(),
                Some(function_item.clone()),
            )?;
        }
    }
    Ok(())
}

fn read_function_parameters(
    file_item: &mut Document,
    params_node: Option<Node>,
    text: String,
    function_item: Arc<SPItem>,
) -> Result<(), Utf8Error> {
    if params_node.is_none() {
        return Ok(());
    }
    let params_node = params_node.unwrap();
    let mut cursor = params_node.walk();
    for child in params_node.children(&mut cursor) {
        let kind = child.kind();
        if kind != "argument_declaration" {
            continue;
        }
        let name_node = child.child_by_field_name("name");
        let type_node = child.child_by_field_name("type");
        let mut storage_class: Vec<VariableStorageClass> = vec![];
        let mut sub_cursor = child.walk();
        for sub_child in child.children(&mut sub_cursor) {
            let sub_child_text = sub_child.utf8_text(text.as_bytes())?;
            if sub_child_text == "const" {
                storage_class.push(VariableStorageClass::Const);
            }
        }
        let name_node = name_node.unwrap();
        let name = name_node.utf8_text(&file_item.text.as_bytes())?;

        let type_ = match type_node {
            Some(type_node) => type_node.utf8_text(&file_item.text.as_bytes())?,
            None => "",
        };
        let detail = child.utf8_text(&text.as_bytes())?;
        let variable_item = VariableItem {
            name: name.to_string(),
            type_: type_.to_string(),
            range: ts_range_to_lsp_range(&name_node.range()),
            description: Description::default(),
            uri: file_item.uri.clone(),
            detail: detail.to_string(),
            visibility: vec![],
            storage_class,
            parent: Some(function_item.clone()),
            references: vec![],
        };
        let variable_item = Arc::new(SPItem::Variable(variable_item));
        file_item.sp_items.push(variable_item);
    }

    Ok(())
}
