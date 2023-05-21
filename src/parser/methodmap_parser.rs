use std::{
    str::Utf8Error,
    sync::{Arc, RwLock},
};

use tree_sitter::Node;

use crate::{
    document::{Document, Walker},
    spitem::{methodmap_item::MethodmapItem, SPItem},
    utils::ts_range_to_lsp_range,
};

impl Document {
    pub(crate) fn parse_methodmap(
        &mut self,
        node: &mut Node,
        walker: &mut Walker,
    ) -> Result<(), Utf8Error> {
        let name_node = node.child_by_field_name("name").unwrap();
        let name = name_node
            .utf8_text(self.preprocessed_text.as_bytes())?
            .to_string();
        let inherit_node = node.child_by_field_name("inherits");
        let inherit = match inherit_node {
            Some(inherit_node) => Some(
                inherit_node
                    .utf8_text(self.preprocessed_text.as_bytes())
                    .unwrap()
                    .trim()
                    .to_string(),
            ),
            None => None,
        };

        let range = ts_range_to_lsp_range(&name_node.range());
        let full_range = ts_range_to_lsp_range(&node.range());
        let methodmap_item = MethodmapItem {
            name,
            range,
            v_range: self.build_v_range(&range),
            full_range,
            v_full_range: self.build_v_range(&full_range),
            parent: None,
            description: walker.find_doc(node.start_position().row, false)?,
            uri: self.uri.clone(),
            references: vec![],
            tmp_parent: inherit,
            children: vec![],
        };

        let methodmap_item = Arc::new(RwLock::new(SPItem::Methodmap(methodmap_item)));
        read_methodmap_members(self, node, methodmap_item.clone(), walker);
        self.sp_items.push(methodmap_item.clone());
        self.declarations
            .insert(methodmap_item.clone().read().unwrap().key(), methodmap_item);

        Ok(())
    }
}

fn read_methodmap_members(
    document: &mut Document,
    node: &Node,
    methodmap_item: Arc<RwLock<SPItem>>,
    walker: &mut Walker,
) {
    let mut cursor = node.walk();
    for mut child in node.children(&mut cursor) {
        match child.kind() {
            "methodmap_method"
            | "methodmap_method_constructor"
            | "methodmap_method_destructor"
            | "methodmap_native"
            | "methodmap_native_constructor"
            | "methodmap_native_destructor" => {
                document
                    .parse_function(&child, walker, Some(methodmap_item.clone()))
                    .unwrap();
            }
            "methodmap_property" => {
                document
                    .parse_property(&mut child, walker, methodmap_item.clone())
                    .unwrap();
            }
            "comment" => walker.push_comment(child, &document.preprocessed_text),
            "preproc_pragma" => walker.push_deprecated(child, &document.preprocessed_text),
            _ => {}
        }
    }
}
