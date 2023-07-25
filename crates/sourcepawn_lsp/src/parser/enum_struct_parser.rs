use std::sync::{Arc, RwLock};

use anyhow::Context;
use tree_sitter::Node;

use crate::{
    document::{Document, Walker},
    providers::hover::description::Description,
    spitem::{enum_struct_item::EnumStructItem, variable_item::VariableItem, SPItem},
    utils::ts_range_to_lsp_range,
};

impl Document {
    pub(crate) fn parse_enum_struct(
        &mut self,
        node: &mut Node,
        walker: &mut Walker,
    ) -> anyhow::Result<()> {
        let name_node = node
            .child_by_field_name("name")
            .context("Enum struct does not have a name field.")?;
        let name = name_node
            .utf8_text(self.preprocessed_text.as_bytes())?
            .to_string();

        let description = walker
            .find_doc(node.start_position().row, false)
            .unwrap_or_default();

        let range = ts_range_to_lsp_range(&name_node.range());
        let full_range = ts_range_to_lsp_range(&node.range());
        let enum_struct_item = EnumStructItem {
            name,
            range,
            v_range: self.build_v_range(&range),
            full_range,
            v_full_range: self.build_v_range(&full_range),
            description,
            uri: self.uri.clone(),
            references: vec![],
            children: vec![],
        };

        let enum_struct_item = Arc::new(RwLock::new(SPItem::EnumStruct(enum_struct_item)));
        self.parse_enum_struct_members(node, enum_struct_item.clone(), walker);
        self.sp_items.push(enum_struct_item.clone());
        self.declarations.insert(
            enum_struct_item.clone().read().unwrap().key(),
            enum_struct_item,
        );

        Ok(())
    }

    fn parse_enum_struct_members(
        &mut self,
        node: &Node,
        enum_struct_item: Arc<RwLock<SPItem>>,
        walker: &mut Walker,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "enum_struct_field" => {
                    let _ = self.parse_enum_struct_field(&child, &enum_struct_item);
                }
                "enum_struct_method" => {
                    let _ = self.parse_function(&child, walker, Some(enum_struct_item.clone()));
                }
                "comment" => walker.push_comment(child, &self.preprocessed_text),
                "preproc_pragma" => {
                    let _ = walker.push_deprecated(child, &self.preprocessed_text);
                }
                _ => {}
            }
        }
    }

    fn parse_enum_struct_field(
        &mut self,
        node: &Node,
        enum_struct_item: &Arc<RwLock<SPItem>>,
    ) -> anyhow::Result<()> {
        // Name of the enum struct field
        let name_node = node
            .child_by_field_name("name")
            .context("Enum struct field does not have a name field.")?;
        let name = name_node.utf8_text(self.preprocessed_text.as_bytes())?;

        let type_node = node
            .child_by_field_name("type")
            .context("Enum struct field does not have a type field.")?;
        let type_ = type_node.utf8_text(self.preprocessed_text.as_bytes())?;

        let mut dimensions = vec![];
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let kind = child.kind();
            match kind {
                "fixed_dimension" | "dimension" => {
                    let dimension_text = child.utf8_text(self.preprocessed_text.as_bytes())?;
                    dimensions.push(dimension_text.to_string());
                }
                _ => {
                    continue;
                }
            }
        }

        let range = ts_range_to_lsp_range(&name_node.range());
        let enum_struct_field_item = VariableItem {
            name: name.to_string(),
            type_: type_.to_string(),
            range,
            v_range: self.build_v_range(&range),
            description: Description::default(),
            uri: self.uri.clone(),
            detail: format!("{} {}{}", type_, name, dimensions.join("")),
            visibility: vec![],
            storage_class: vec![],
            parent: Some(Arc::downgrade(enum_struct_item)),
            references: vec![],
        };

        let enum_struct_field_item =
            Arc::new(RwLock::new(SPItem::Variable(enum_struct_field_item)));

        enum_struct_item
            .write()
            .unwrap()
            .push_child(enum_struct_field_item);

        Ok(())
    }
}
