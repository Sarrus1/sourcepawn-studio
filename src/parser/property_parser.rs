use std::{
    str::Utf8Error,
    sync::{Arc, RwLock},
};

use tree_sitter::Node;

use crate::{
    document::{Document, Walker},
    spitem::{property_item::PropertyItem, SPItem},
    utils::ts_range_to_lsp_range,
};

impl Document {
    pub fn parse_property(
        &mut self,
        node: &mut Node,
        walker: &mut Walker,
        parent: Arc<RwLock<SPItem>>,
    ) -> Result<(), Utf8Error> {
        let name_node = node.child_by_field_name("name").unwrap();
        let name = name_node
            .utf8_text(self.preprocessed_text.as_bytes())?
            .to_string();
        let type_node = node.child_by_field_name("type").unwrap();
        let type_ = type_node
            .utf8_text(self.preprocessed_text.as_bytes())
            .unwrap();

        let property_item = PropertyItem {
            name,
            range: ts_range_to_lsp_range(&name_node.range()),
            full_range: ts_range_to_lsp_range(&node.range()),
            type_: type_.to_string(),
            description: walker.find_doc(node.start_position().row, false)?,
            uri: self.uri.clone(),
            references: vec![],
            parent: Arc::downgrade(&parent),
        };

        let property_item = Arc::new(RwLock::new(SPItem::Property(property_item)));
        parent.write().unwrap().push_child(property_item.clone());
        self.declarations
            .insert(property_item.clone().read().unwrap().key(), property_item);

        // TODO: Add getter and setter parsing.
        Ok(())
    }
}
