use anyhow::Context;
use parking_lot::RwLock;
use std::sync::Arc;
use syntax::{define_item::DefineItem, utils::ts_range_to_lsp_range, SPItem};
use tree_sitter::Node;

use crate::Parser;

impl<'a> Parser<'a> {
    pub fn parse_macro(&mut self, node: &mut Node) -> anyhow::Result<()> {
        let name_node = node
            .child_by_field_name("name")
            .context("Define does not have a name field.")?;
        let name = name_node.utf8_text(self.source.as_bytes())?.to_string();
        let range = ts_range_to_lsp_range(&name_node.range());
        let full_range = ts_range_to_lsp_range(&node.range());
        let define_item = DefineItem {
            name,
            range,
            v_range: self.build_v_range(&range),
            full_range,
            v_full_range: self.build_v_range(&full_range),
            value: "".to_string(),
            description: self.find_doc(node.start_position().row, true)?,
            uri: self.uri.clone(),
            file_id: self.file_id,
            references: vec![],
        };

        let define_item = Arc::new(RwLock::new(SPItem::Define(define_item)));
        self.sp_items.push(define_item.clone());
        self.declarations
            .insert(define_item.clone().read().key(), define_item);

        Ok(())
    }
}
