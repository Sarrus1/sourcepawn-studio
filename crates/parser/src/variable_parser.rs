use anyhow::Context;
use parking_lot::RwLock;
use std::sync::Arc;
use syntax::{
    description::Description,
    utils::ts_range_to_lsp_range,
    variable_item::{VariableItem, VariableStorageClass, VariableVisibility},
    SPItem,
};
use tree_sitter::Node;

use crate::Parser;

impl<'a> Parser<'a> {
    pub fn parse_variable(
        &mut self,
        node: &mut Node,
        parent: Option<Arc<RwLock<SPItem>>>,
    ) -> anyhow::Result<()> {
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
                    let _ = self.parse_variable_visibility(child, &mut visibility);
                }
                "variable_storage_class" => {
                    let _ = self.parse_variable_storage_class(child, &mut storage_class);
                }
                "variable_declaration" | "old_variable_declaration" => {
                    let _ = self.parse_variable_declaration(
                        child,
                        type_node,
                        &visibility,
                        &storage_class,
                        &parent,
                    );
                }
                _ => (),
            }
        }

        Ok(())
    }

    fn parse_variable_visibility(
        &mut self,
        child: Node,
        visibility: &mut Vec<VariableVisibility>,
    ) -> Result<(), anyhow::Error> {
        let visibility_text = child.utf8_text(self.source.as_bytes())?;
        if visibility_text.contains("stock") {
            visibility.push(VariableVisibility::Stock);
        }
        if visibility_text.contains("public") {
            visibility.push(VariableVisibility::Public);
        }

        Ok(())
    }

    fn parse_variable_storage_class(
        &mut self,
        child: Node,
        storage_class: &mut Vec<VariableStorageClass>,
    ) -> Result<(), anyhow::Error> {
        let storage_class_text = child.utf8_text(self.source.as_bytes())?;
        if storage_class_text.contains("const") {
            storage_class.push(VariableStorageClass::Const);
        }
        if storage_class_text.contains("static") {
            storage_class.push(VariableStorageClass::Static);
        }

        Ok(())
    }

    fn parse_variable_declaration(
        &mut self,
        child: Node,
        type_node: Option<Node>,
        visibility: &[VariableVisibility],
        storage_class: &[VariableStorageClass],
        parent: &Option<Arc<RwLock<SPItem>>>,
    ) -> Result<(), anyhow::Error> {
        let name_node = child
            .child_by_field_name("name")
            .context("Variable declaration does not have a name.")?;
        let name = name_node.utf8_text(self.source.as_bytes())?.to_string();
        let mut dimensions: Vec<String> = vec![];
        let mut cursor = child.walk();
        for sub_child in child.children(&mut cursor) {
            let kind = sub_child.kind();
            match kind {
                "fixed_dimension" | "dimension" => {
                    let dimension_text = sub_child.utf8_text(self.source.as_bytes())?;
                    dimensions.push(dimension_text.to_string());
                }
                _ => {
                    continue;
                }
            }
        }
        let type_ = match type_node {
            Some(type_node) => Some(type_node.utf8_text(self.source.as_bytes())?.to_string()),
            None => None,
        };
        let range = ts_range_to_lsp_range(&name_node.range());
        let variable_item = VariableItem {
            name,
            type_: type_.unwrap_or_default(),
            range,
            v_range: self.build_v_range(&range),
            description: Description::default(),
            uri: self.uri.clone(),
            file_id: self.file_id,
            detail: "".to_string(),
            visibility: visibility.to_vec(),
            storage_class: storage_class.to_vec(),
            parent: parent.as_ref().map(Arc::downgrade),
            references: vec![],
        };
        let variable_item = Arc::new(RwLock::new(SPItem::Variable(variable_item)));
        if let Some(parent) = parent {
            // Don't add the variable item as a declaration if it's a local variable.
            parent.write().push_child(variable_item);
        } else {
            self.sp_items.push(variable_item.clone());
            self.declarations
                .insert(variable_item.clone().read().key(), variable_item);
        }

        Ok(())
    }
}
