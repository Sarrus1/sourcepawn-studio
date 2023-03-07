use std::{
    str::Utf8Error,
    sync::{Arc, RwLock},
};

use fxhash::FxHashSet;
use regex::Regex;
use tree_sitter::{Node, QueryCursor};

use crate::{
    document::{find_doc, Document, Walker},
    providers::hover::description::Description,
    spitem::{
        function_item::{FunctionDefinitionType, FunctionItem, FunctionVisibility},
        parameter::Parameter,
        variable_item::{VariableItem, VariableStorageClass},
        SPItem,
    },
    utils::ts_range_to_lsp_range,
};

use super::{typedef_parser::parse_argument_type, VARIABLE_QUERY};

impl Document {
    pub fn parse_function(
        &mut self,
        node: &Node,
        walker: &mut Walker,
        parent: Option<Arc<RwLock<SPItem>>>,
    ) -> Result<(), Utf8Error> {
        // Name of the function
        let name_node = node.child_by_field_name("name");
        // Return type of the function
        let type_node = node.child_by_field_name("returnType");
        // Visibility of the function (public, static, stock)
        let mut visibility_node: Option<Node> = None;
        // Parameters of the declaration
        let mut argument_declarations_node: Option<Node> = None;
        // Type of function definition ("native" or "forward")
        let mut definition_type_node: Option<Node> = None;

        let mut block_node: Option<Node> = None;

        let mut visibility = FxHashSet::default();

        let mut definition_type = FunctionDefinitionType::None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let kind = child.kind();
            match kind {
                "function_visibility" => {
                    visibility_node = Some(child);
                }
                "argument_declarations" => {
                    argument_declarations_node = Some(child);
                }
                "function_definition_type" => {
                    definition_type_node = Some(child);
                }
                "block" => {
                    block_node = Some(child);
                }
                "static" => {
                    visibility.insert(FunctionVisibility::Static);
                }
                "public" => {
                    visibility.insert(FunctionVisibility::Public);
                }
                "native" => {
                    definition_type = FunctionDefinitionType::Native;
                }
                _ => {}
            }
        }

        if name_node.is_none() {
            // A function always has a name.
            return Ok(());
        }
        let name_node = name_node.unwrap();
        let name = name_node.utf8_text(self.text.as_bytes())?.to_string();

        let mut type_ = Ok("");
        if let Some(type_node) = type_node {
            type_ = type_node.utf8_text(self.text.as_bytes());
        }

        if visibility_node.is_some() {
            let visibility_text = visibility_node.unwrap().utf8_text(self.text.as_bytes())?;
            if visibility_text.contains("stock") {
                visibility.insert(FunctionVisibility::Stock);
            }
            if visibility_text.contains("public") {
                visibility.insert(FunctionVisibility::Public);
            }
            if visibility_text.contains("static") {
                visibility.insert(FunctionVisibility::Static);
            }
        }

        if definition_type_node.is_some() {
            definition_type = match definition_type_node
                .unwrap()
                .utf8_text(self.text.as_bytes())?
            {
                "forward" => FunctionDefinitionType::Forward,
                "native" => FunctionDefinitionType::Native,
                _ => FunctionDefinitionType::None,
            }
        }

        let documentation = find_doc(walker, node.start_position().row)?;

        let function_item = FunctionItem {
            name: name.clone(),
            type_: type_?.to_string(),
            range: ts_range_to_lsp_range(&name_node.range()),
            full_range: ts_range_to_lsp_range(&node.range()),
            description: documentation.clone(),
            uri: self.uri.clone(),
            detail: build_detail(
                self,
                &name,
                type_,
                argument_declarations_node,
                visibility_node,
                definition_type_node,
            )?,
            visibility,
            definition_type,
            references: vec![],
            parent: parent.as_ref().map(Arc::downgrade),
            params: vec![],
            children: vec![],
        };

        let function_item = Arc::new(RwLock::new(SPItem::Function(function_item)));
        if let Some(block_node) = block_node {
            read_body_variables(
                self,
                block_node,
                self.text.to_string(),
                function_item.clone(),
            )?
        }
        read_function_parameters(
            self,
            documentation,
            argument_declarations_node,
            self.text.to_string(),
            function_item.clone(),
        )?;
        if let Some(parent) = &parent {
            parent.write().unwrap().push_child(function_item.clone());
        } else {
            self.sp_items.push(function_item.clone());
        }
        self.declarations
            .insert(function_item.clone().read().unwrap().key(), function_item);

        Ok(())
    }
}

fn build_detail(
    document: &Document,
    name: &str,
    type_: Result<&str, Utf8Error>,
    params_node: Option<Node>,
    visibility_node: Option<Node>,
    definition_type_node: Option<Node>,
) -> Result<String, Utf8Error> {
    let mut detail = format!("{} {}", type_?, name);
    if let Some(params_node) = params_node {
        detail.push_str(params_node.utf8_text(document.text.as_bytes()).unwrap());
    }
    if visibility_node.is_some() {
        detail = format!(
            "{} {}",
            visibility_node
                .unwrap()
                .utf8_text(document.text.as_bytes())?,
            detail
        )
    }

    if definition_type_node.is_some() {
        detail = format!(
            "{} {}",
            definition_type_node
                .unwrap()
                .utf8_text(document.text.as_bytes())?,
            detail
        );
    }

    Ok(detail.trim().to_string())
}

fn read_body_variables(
    document: &mut Document,
    block_node: Node,
    text: String,
    function_item: Arc<RwLock<SPItem>>,
) -> Result<(), Utf8Error> {
    let mut cursor = QueryCursor::new();
    let matches = cursor.captures(&VARIABLE_QUERY, block_node, text.as_bytes());
    for (match_, _) in matches {
        for capture in match_.captures.iter() {
            document.parse_variable(&mut capture.node.clone(), Some(function_item.clone()))?;
        }
    }
    Ok(())
}

fn read_function_parameters(
    document: &mut Document,
    documentation: Description,
    argument_declarations_node: Option<Node>,
    text: String,
    function_item: Arc<RwLock<SPItem>>,
) -> Result<(), Utf8Error> {
    if argument_declarations_node.is_none() {
        return Ok(());
    }

    let argument_declarations_node = argument_declarations_node.unwrap();
    let mut cursor = argument_declarations_node.walk();
    for child in argument_declarations_node.children(&mut cursor) {
        if child.kind() != "argument_declaration" {
            continue;
        }
        let name_node = child.child_by_field_name("name");
        let type_node = child.child_by_field_name("type");
        let mut is_const = false;
        let mut dimensions = vec![];
        let mut storage_class: Vec<VariableStorageClass> = vec![];
        let mut sub_cursor = child.walk();
        for sub_child in child.children(&mut sub_cursor) {
            match sub_child.kind() {
                "const" => {
                    is_const = true;
                    storage_class.push(VariableStorageClass::Const);
                }
                "dimension" | "fixed_dimension" => {
                    let dimension = sub_child.utf8_text(text.as_bytes())?;
                    dimensions.push(dimension.to_string());
                }
                _ => {}
            }
        }
        let name_node = name_node.unwrap();
        let name = name_node.utf8_text(document.text.as_bytes())?;

        let type_ = match type_node {
            Some(type_node) => type_node.utf8_text(document.text.as_bytes())?,
            None => "",
        };
        let detail = child.utf8_text(text.as_bytes())?;
        let description = Description {
            text: match extract_param_doc(name, &documentation) {
                Some(text) => text,
                None => "".to_string(),
            },
            deprecated: None,
        };
        let variable_item = VariableItem {
            name: name.to_string(),
            type_: type_.to_string(),
            range: ts_range_to_lsp_range(&name_node.range()),
            description: description.clone(),
            uri: document.uri.clone(),
            detail: detail.to_string(),
            visibility: vec![],
            storage_class,
            parent: Some(Arc::downgrade(&function_item)),
            references: vec![],
        };
        let variable_item = Arc::new(RwLock::new(SPItem::Variable(variable_item)));
        function_item.write().unwrap().push_child(variable_item);

        let parameter = Parameter {
            name: name.to_string(),
            is_const,
            type_: parse_argument_type(document, type_node),
            description,
            dimensions,
        };
        function_item
            .write()
            .unwrap()
            .push_param(Arc::new(RwLock::new(parameter)));
    }

    Ok(())
}

pub(crate) fn extract_param_doc(name: &str, documentation: &Description) -> Option<String> {
    let re = Regex::new(&format!("@param\\s+(?:\\b{}\\b)([^@]+)", name)).unwrap();
    if let Some(caps) = re.captures(&documentation.text) {
        if let Some(text) = caps.get(1) {
            return Some(text.as_str().replace('*', "").trim().to_string());
        }
    };

    None
}
