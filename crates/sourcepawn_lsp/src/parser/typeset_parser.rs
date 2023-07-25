use std::sync::{Arc, RwLock};

use anyhow::Context;
use tree_sitter::Node;

use crate::{
    document::{Document, Walker},
    spitem::{typedef_item::TypedefItem, typeset_item::TypesetItem, SPItem},
    utils::ts_range_to_lsp_range,
};

impl Document {
    pub fn parse_typeset(&mut self, node: &Node, walker: &mut Walker) -> anyhow::Result<()> {
        let name_node = node
            .child_by_field_name("name")
            .context("Typeset name is empty.")?;
        let name = name_node
            .utf8_text(self.preprocessed_text.as_bytes())?
            .to_string();

        let description = walker
            .find_doc(node.start_position().row, false)
            .unwrap_or_default();

        let mut children = vec![];
        let mut cursor = node.walk();
        let mut counter = -1;
        for child in node.children(&mut cursor) {
            match child.kind() {
                "comment" => walker.push_comment(child, &self.preprocessed_text),
                "preproc_pragma" => {
                    let _ = walker.push_deprecated(child, &self.preprocessed_text);
                }
                "typedef_expression" => {
                    let _ = self.parse_typeset_expression(
                        &mut counter,
                        child,
                        walker,
                        node,
                        name_node,
                        &mut children,
                    );
                }
                _ => {}
            }
        }

        let range = ts_range_to_lsp_range(&name_node.range());
        let full_range = ts_range_to_lsp_range(&node.range());
        let typeset_item = TypesetItem {
            name,
            range,
            v_range: self.build_v_range(&range),
            full_range,
            v_full_range: self.build_v_range(&full_range),
            description,
            uri: self.uri.clone(),
            references: vec![],
            children,
        };

        let typeset_item = Arc::new(RwLock::new(SPItem::Typeset(typeset_item)));
        self.sp_items.push(typeset_item.clone());
        self.declarations
            .insert(typeset_item.clone().read().unwrap().key(), typeset_item);

        Ok(())
    }

    fn parse_typeset_expression(
        &mut self,
        counter: &mut i32,
        child: Node,
        walker: &mut Walker,
        node: &Node,
        name_node: Node,
        children: &mut Vec<Arc<RwLock<SPItem>>>,
    ) -> Result<(), anyhow::Error> {
        let name = name_node
            .utf8_text(self.preprocessed_text.as_bytes())?
            .to_string();
        *counter += 1;
        let mut argument_declarations_node = None;
        let type_node = child.child_by_field_name("returnType");
        let mut sub_cursor = child.walk();
        for sub_child in child.children(&mut sub_cursor) {
            if sub_child.kind() == "argument_declarations" {
                argument_declarations_node = Some(sub_child)
            }
        }
        let type_ = match type_node {
            Some(type_node) => Some(
                type_node
                    .utf8_text(self.preprocessed_text.as_bytes())?
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
            name: format!("{}{}", name, counter),
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
        children.push(typedef_item);

        Ok(())
    }
}
