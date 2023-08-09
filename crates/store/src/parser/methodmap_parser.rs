use parking_lot::RwLock;
use std::sync::Arc;
use syntax::{methodmap_item::MethodmapItem, utils::ts_range_to_lsp_range, SPItem};

use anyhow::Context;
use tree_sitter::Node;

use crate::document::{Document, Walker};

impl Document {
    pub(crate) fn parse_methodmap(
        &mut self,
        node: &mut Node,
        walker: &mut Walker,
    ) -> anyhow::Result<()> {
        let name_node = node
            .child_by_field_name("name")
            .context("Methodmap does not have a name field.")?;
        let name = name_node
            .utf8_text(self.preprocessed_text.as_bytes())?
            .to_string();
        let inherit_node = node.child_by_field_name("inherits");
        let inherit = self.get_inherit_string(inherit_node);

        let range = ts_range_to_lsp_range(&name_node.range());
        let full_range = ts_range_to_lsp_range(&node.range());
        let methodmap_item = MethodmapItem {
            name,
            range,
            v_range: self.build_v_range(&range),
            full_range,
            v_full_range: self.build_v_range(&full_range),
            parent: None,
            description: walker
                .find_doc(node.start_position().row, false)
                .unwrap_or_default(),
            uri: self.uri.clone(),
            references: vec![],
            tmp_parent: inherit,
            children: vec![],
        };

        let methodmap_item = Arc::new(RwLock::new(SPItem::Methodmap(methodmap_item)));
        let _ = self.read_methodmap_members(node, methodmap_item.clone(), walker);
        self.sp_items.push(methodmap_item.clone());
        self.declarations
            .insert(methodmap_item.clone().read().key(), methodmap_item);

        Ok(())
    }

    fn get_inherit_string(&self, inherit_node: Option<Node>) -> Option<String> {
        Some(
            inherit_node?
                .utf8_text(self.preprocessed_text.as_bytes())
                .ok()?
                .trim()
                .to_string(),
        )
    }

    fn read_methodmap_members(
        &mut self,
        node: &Node,
        methodmap_item: Arc<RwLock<SPItem>>,
        walker: &mut Walker,
    ) -> anyhow::Result<()> {
        let mut cursor = node.walk();
        for mut child in node.children(&mut cursor) {
            match child.kind() {
                "methodmap_method"
                | "methodmap_method_constructor"
                | "methodmap_method_destructor"
                | "methodmap_native"
                | "methodmap_native_constructor"
                | "methodmap_native_destructor" => {
                    let _ = self.parse_function(&child, walker, Some(methodmap_item.clone()));
                }
                "methodmap_property" => {
                    let _ = self.parse_property(&mut child, walker, methodmap_item.clone());
                }
                "comment" => walker.push_comment(child, &self.preprocessed_text),
                "preproc_pragma" => {
                    let _ = walker.push_deprecated(child, &self.preprocessed_text);
                }
                _ => {}
            }
        }

        Ok(())
    }
}
