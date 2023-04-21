use std::{
    str::Utf8Error,
    sync::{Arc, RwLock},
};

use tree_sitter::Node;

use crate::{
    document::{Document, Walker},
    spitem::{define_item::DefineItem, SPItem},
    utils::ts_range_to_lsp_range,
};

impl Document {
    pub(crate) fn parse_define(
        &mut self,
        node: &mut Node,
        walker: &mut Walker,
    ) -> Result<(), Utf8Error> {
        let name_node = node.child_by_field_name("name").unwrap();
        let name = name_node
            .utf8_text(self.preprocessed_text.as_bytes())?
            .to_string();
        let value_node = node.child_by_field_name("value");
        let value = match value_node {
            Some(value_node) => value_node
                .utf8_text(self.preprocessed_text.as_bytes())
                .unwrap()
                .trim(),
            None => "",
        };

        let define_item = DefineItem {
            name,
            range: ts_range_to_lsp_range(&name_node.range()),
            full_range: ts_range_to_lsp_range(&node.range()),
            value: value.to_string(),
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
