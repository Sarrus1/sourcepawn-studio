use anyhow::Context;
use fxhash::FxHashSet;
use parking_lot::RwLock;
use regex::Regex;
use std::{str::Utf8Error, sync::Arc};
use tree_sitter::{Node, QueryCursor};

use super::VARIABLE_QUERY;
use crate::{
    document::{Document, Walker},
    providers::hover::description::Description,
    spitem::{
        function_item::{FunctionDefinitionType, FunctionItem, FunctionVisibility},
        parameter::Parameter,
        variable_item::{VariableItem, VariableStorageClass},
        SPItem,
    },
    utils::ts_range_to_lsp_range,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct FunctionAttributes<'a> {
    name: String,
    type_: Option<String>,
    visibility: FxHashSet<FunctionVisibility>,
    definition_type: FunctionDefinitionType,
    block_node: Option<Node<'a>>,
    argument_declarations_node: Option<Node<'a>>,

    /// Visibility of the function (public, static, stock)
    visibility_node: Option<Node<'a>>,

    /// Type of function definition ("native" or "forward")
    definition_type_node: Option<Node<'a>>,
}

impl<'a> FunctionAttributes<'a> {
    fn populate(&mut self, node: &'a Node, document: &Document) -> Result<(), Utf8Error> {
        // Return type of the function
        let type_node = node.child_by_field_name("returnType");
        self.type_ = match type_node {
            Some(type_node) => Some(
                type_node
                    .utf8_text(document.preprocessed_text.as_bytes())?
                    .to_string(),
            ),
            None => None,
        };

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let kind = child.kind();
            match kind {
                "function_visibility" => {
                    self.visibility_node = Some(child);
                }
                "argument_declarations" => {
                    self.argument_declarations_node = Some(child);
                }
                "function_definition_type" => {
                    self.definition_type_node = Some(child);
                }
                "block" => {
                    self.block_node = Some(child);
                }
                "static" => {
                    self.visibility.insert(FunctionVisibility::Static);
                }
                "public" => {
                    self.visibility.insert(FunctionVisibility::Public);
                }
                "native" => {
                    self.definition_type = FunctionDefinitionType::Native;
                }
                _ => {}
            }
        }

        if let Some(visibility_node) = self.visibility_node {
            let visibility_text =
                visibility_node.utf8_text(document.preprocessed_text.as_bytes())?;
            if visibility_text.contains("stock") {
                self.visibility.insert(FunctionVisibility::Stock);
            }
            if visibility_text.contains("public") {
                self.visibility.insert(FunctionVisibility::Public);
            }
            if visibility_text.contains("static") {
                self.visibility.insert(FunctionVisibility::Static);
            }
        }

        if let Some(definition_type_node) = self.definition_type_node {
            self.definition_type =
                match definition_type_node.utf8_text(document.preprocessed_text.as_bytes())? {
                    "forward" => FunctionDefinitionType::Forward,
                    "native" => FunctionDefinitionType::Native,
                    _ => FunctionDefinitionType::None,
                }
        }

        Ok(())
    }

    fn build_detail(&self, document: &Document) -> Result<String, Utf8Error> {
        let mut detail = format!("{} {}", self.type_(), self.name);
        if let Some(params_node) = self.argument_declarations_node {
            detail.push_str(
                params_node
                    .utf8_text(document.preprocessed_text.as_bytes())
                    .unwrap(),
            );
        }
        if let Some(visibility_node) = self.visibility_node {
            detail = format!(
                "{} {}",
                visibility_node.utf8_text(document.preprocessed_text.as_bytes())?,
                detail
            );
        }

        if let Some(definition_type_node) = self.definition_type_node {
            detail = format!(
                "{} {}",
                definition_type_node.utf8_text(document.preprocessed_text.as_bytes())?,
                detail
            );
        }

        Ok(detail.trim().to_string())
    }

    fn type_(&self) -> String {
        self.type_.clone().unwrap_or_default()
    }
}

impl Document {
    pub(crate) fn parse_function(
        &mut self,
        node: &Node,
        walker: &mut Walker,
        parent: Option<Arc<RwLock<SPItem>>>,
    ) -> anyhow::Result<()> {
        // Name of the function
        let name_node = node
            .child_by_field_name("name")
            .context("Function does not have a name field.")?;
        let name = name_node
            .utf8_text(self.preprocessed_text.as_bytes())?
            .to_string();

        let mut attributes = FunctionAttributes {
            name,
            ..Default::default()
        };

        let _ = attributes.populate(node, self);

        let description = walker
            .find_doc(node.start_position().row, false)
            .unwrap_or_default();

        let range = ts_range_to_lsp_range(&name_node.range());
        let full_range = ts_range_to_lsp_range(&node.range());
        let function_item = FunctionItem {
            name: attributes.name.clone(),
            type_: attributes.type_(),
            range,
            v_range: self.build_v_range(&range),
            full_range,
            v_full_range: self.build_v_range(&full_range),
            description: description.clone(),
            uri: self.uri.clone(),
            detail: attributes.build_detail(self).unwrap_or_default(),
            visibility: attributes.visibility,
            definition_type: attributes.definition_type,
            references: vec![],
            parent: parent.as_ref().map(Arc::downgrade),
            params: vec![],
            children: vec![],
        };

        let function_item = Arc::new(RwLock::new(SPItem::Function(function_item)));
        if let Some(block_node) = attributes.block_node {
            let _ = self.read_body_variables(block_node, function_item.clone());
        }
        let _ = self.read_function_parameters(
            description,
            attributes.argument_declarations_node,
            function_item.clone(),
        );
        if let Some(parent) = &parent {
            parent.write().push_child(function_item.clone());
        } else {
            self.sp_items.push(function_item.clone());
        }
        self.declarations
            .insert(function_item.clone().read().key(), function_item);

        Ok(())
    }

    fn read_function_parameters(
        &mut self,
        description: Description,
        argument_declarations_node: Option<Node>,
        function_item: Arc<RwLock<SPItem>>,
    ) -> anyhow::Result<()> {
        if argument_declarations_node.is_none() {
            return Ok(());
        }

        let argument_declarations_node =
            argument_declarations_node.context("No argument declarations node")?;
        let mut cursor = argument_declarations_node.walk();
        for child in argument_declarations_node.children(&mut cursor) {
            let _ = self.read_function_parameter(&child, &description, &function_item);
        }

        Ok(())
    }

    fn read_function_parameter(
        &self,
        child: &Node,
        description: &Description,
        function_item: &Arc<RwLock<SPItem>>,
    ) -> anyhow::Result<()> {
        if child.kind() != "argument_declaration" {
            return Ok(());
        }
        let name_node = child
            .child_by_field_name("name")
            .context("Function parameter does not have a name.")?;
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
                    let dimension = sub_child.utf8_text(self.preprocessed_text.as_bytes())?;
                    dimensions.push(dimension.to_string());
                }
                _ => {}
            }
        }
        let name = name_node.utf8_text(self.preprocessed_text.as_bytes())?;

        let type_ = match type_node {
            Some(type_node) => type_node.utf8_text(self.preprocessed_text.as_bytes())?,
            None => "",
        };
        let detail = child.utf8_text(self.preprocessed_text.as_bytes())?;
        let description = Description {
            text: match extract_param_from_desc(name, description) {
                Some(text) => text,
                None => "".to_string(),
            },
            deprecated: None,
        };

        let range = ts_range_to_lsp_range(&name_node.range());
        let variable_item = VariableItem {
            name: name.to_string(),
            type_: type_.to_string(),
            range,
            v_range: self.build_v_range(&range),
            description: description.clone(),
            uri: self.uri.clone(),
            detail: detail.to_string(),
            visibility: vec![],
            storage_class,
            parent: Some(Arc::downgrade(function_item)),
            references: vec![],
        };
        let variable_item = Arc::new(RwLock::new(SPItem::Variable(variable_item)));
        function_item.write().push_child(variable_item);

        let parameter = Parameter {
            name: name.to_string(),
            is_const,
            type_: self.parse_argument_type(type_node),
            description,
            dimensions,
        };
        function_item
            .write()
            .push_param(Arc::new(RwLock::new(parameter)));

        Ok(())
    }

    fn read_body_variables(
        &mut self,
        block_node: Node,
        function_item: Arc<RwLock<SPItem>>,
    ) -> Result<(), Utf8Error> {
        let mut cursor = QueryCursor::new();
        let matches = cursor.captures(
            &VARIABLE_QUERY,
            block_node,
            self.preprocessed_text.as_bytes(),
        );
        let nodes: Vec<_> = matches
            .flat_map(|(match_, _)| match_.captures.iter().map(|capture| capture.node))
            .collect();
        nodes.iter().for_each(|node| {
            let _ = self.parse_variable(&mut node.clone(), Some(function_item.clone()));
        });

        Ok(())
    }
}

pub(crate) fn extract_param_from_desc(name: &str, description: &Description) -> Option<String> {
    let re = Regex::new(&format!("@param\\s+(?:\\b{}\\b)([^@]+)", name)).unwrap();
    if let Some(caps) = re.captures(&description.text) {
        if let Some(text) = caps.get(1) {
            return Some(text.as_str().replace('*', "").trim().to_string());
        }
    };

    None
}
