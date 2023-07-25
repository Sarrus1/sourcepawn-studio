use std::{
    str::Utf8Error,
    sync::{Arc, RwLock},
};

use anyhow::Context;
use tree_sitter::Node;

use crate::{
    document::{Document, Walker},
    providers::hover::description::Description,
    spitem::{
        parameter::{Parameter, Type},
        typedef_item::TypedefItem,
        SPItem,
    },
    utils::ts_range_to_lsp_range,
};

use super::function_parser::extract_param_from_desc;

impl Document {
    pub(crate) fn parse_typedef(&mut self, node: &Node, walker: &mut Walker) -> anyhow::Result<()> {
        // Name of the typedef
        let name_node = node
            .child_by_field_name("name")
            .context("Typedef name is empty.")?;
        let name = name_node
            .utf8_text(self.preprocessed_text.as_bytes())?
            .to_string();

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

        let type_ = match type_node {
            Some(type_node) => Some(
                type_node
                    .utf8_text(self.preprocessed_text.as_bytes())
                    .unwrap_or_default()
                    .to_string(),
            ),
            None => None,
        };
        let description = walker
            .find_doc(node.start_position().row, false)
            .unwrap_or_default();

        let range = ts_range_to_lsp_range(&name_node.range());
        let full_range = ts_range_to_lsp_range(&node.range());
        let typedef_item = TypedefItem {
            name,
            type_: type_.unwrap_or_default(),
            range,
            v_range: self.build_v_range(&range),
            full_range,
            v_full_range: self.build_v_range(&full_range),
            description: description.clone(),
            uri: self.uri.clone(),
            detail: node
                .utf8_text(self.preprocessed_text.as_bytes())
                .unwrap_or_default()
                .to_string(),
            references: vec![],
            params: vec![],
        };

        let typedef_item = Arc::new(RwLock::new(SPItem::Typedef(typedef_item)));
        let _ = self.read_argument_declarations(
            argument_declarations_node,
            typedef_item.clone(),
            description,
        );
        self.sp_items.push(typedef_item.clone());
        self.declarations
            .insert(typedef_item.clone().read().unwrap().key(), typedef_item);

        Ok(())
    }

    pub(super) fn read_argument_declarations(
        &self,
        argument_declarations_node: Option<Node>,
        parent: Arc<RwLock<SPItem>>,
        description: Description,
    ) -> Result<(), Utf8Error> {
        if let Some(argument_declarations_node) = argument_declarations_node {
            let mut cursor = argument_declarations_node.walk();
            for child in argument_declarations_node.children(&mut cursor) {
                match child.kind() {
                    "argument_declaration" => {
                        let _ = self.read_argument_declaration(child, &description, &parent);
                    }
                    "rest_argument" => {
                        // TODO: Handle this
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn read_argument_declaration(
        &self,
        child: Node,
        description: &Description,
        parent: &Arc<RwLock<SPItem>>,
    ) -> anyhow::Result<()> {
        let name_node = child
            .child_by_field_name("name")
            .context("Argument name is empty.")?;
        let argument_type_node = child.child_by_field_name("type");
        let mut is_const = false;
        let mut dimensions = vec![];
        let mut sub_cursor = child.walk();
        for sub_child in child.children(&mut sub_cursor) {
            match sub_child.kind() {
                "const" => is_const = true,
                "dimension" | "fixed_dimension" => {
                    let dimension = sub_child.utf8_text(self.preprocessed_text.as_bytes())?;
                    dimensions.push(dimension.to_string());
                }
                _ => {}
            }
        }
        let name = name_node.utf8_text(self.preprocessed_text.as_bytes())?;
        let parameter = Parameter {
            name: name.to_string(),
            is_const,
            type_: self.parse_argument_type(argument_type_node),
            description: Description {
                text: extract_param_from_desc(name, description).unwrap_or_default(),
                deprecated: None,
            },
            dimensions,
        };
        parent
            .write()
            .unwrap()
            .push_param(Arc::new(RwLock::new(parameter)));
        Ok(())
    }

    pub(crate) fn parse_argument_type(&self, argument_type_node: Option<Node>) -> Option<Type> {
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
                        .utf8_text(self.preprocessed_text.as_bytes())
                        .ok()?
                        .to_string();
                }
                "&" => type_.is_pointer = true,
                "dimension" | "fixed_dimension" => {
                    type_.dimensions.push(
                        child
                            .utf8_text(self.preprocessed_text.as_bytes())
                            .ok()?
                            .to_string(),
                    );
                }
                _ => {}
            }
        }

        Some(type_)
    }
}
