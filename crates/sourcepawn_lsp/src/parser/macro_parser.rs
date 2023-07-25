use std::sync::{Arc, RwLock};

use anyhow::Context;
use tree_sitter::Node;

use crate::{
    document::{Document, Walker},
    spitem::{define_item::DefineItem, SPItem},
    utils::ts_range_to_lsp_range,
};

impl Document {
    pub(crate) fn parse_macro(
        &mut self,
        node: &mut Node,
        walker: &mut Walker,
    ) -> anyhow::Result<()> {
        let name_node = node
            .child_by_field_name("name")
            .context("Define does not have a name field.")?;
        let name = name_node
            .utf8_text(self.preprocessed_text.as_bytes())?
            .to_string();
        let range = ts_range_to_lsp_range(&name_node.range());
        let full_range = ts_range_to_lsp_range(&node.range());
        let define_item = DefineItem {
            name,
            range,
            v_range: self.build_v_range(&range),
            full_range,
            v_full_range: self.build_v_range(&full_range),
            value: "".to_string(),
            description: walker.find_doc(node.start_position().row, true)?,
            uri: self.uri.clone(),
            references: vec![],
        };

        let define_item = Arc::new(RwLock::new(SPItem::Define(define_item)));
        self.sp_items.push(define_item.clone());
        self.declarations
            .insert(define_item.clone().read().unwrap().key(), define_item);

        Ok(())
    }
}
