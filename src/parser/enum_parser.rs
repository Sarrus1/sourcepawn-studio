use std::{
    str::Utf8Error,
    sync::{Arc, RwLock},
};

use anyhow::Context;
use tree_sitter::Node;

use crate::{
    document::{Document, Walker},
    providers::hover::description::Description,
    spitem::{enum_item::EnumItem, enum_member_item::EnumMemberItem, SPItem},
    utils::ts_range_to_lsp_range,
};

use lsp_types::{Position, Range};

impl Document {
    pub(crate) fn parse_enum(
        &mut self,
        node: &mut Node,
        walker: &mut Walker,
    ) -> anyhow::Result<()> {
        let (name, range) = self.get_enum_name_and_range(node, &mut walker.anon_enum_counter)?;
        let description = walker
            .find_doc(node.start_position().row, false)
            .unwrap_or_default();

        let full_range = ts_range_to_lsp_range(&node.range());
        let enum_item = EnumItem {
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

        let mut cursor = node.walk();
        let mut enum_entries: Option<Node> = None;
        for child in node.children(&mut cursor) {
            if child.kind() == "enum_entries" {
                enum_entries = Some(child);
                break;
            }
        }
        let enum_item = Arc::new(RwLock::new(SPItem::Enum(enum_item)));
        if let Some(enum_entries) = enum_entries {
            self.read_enum_members(&enum_entries, enum_item.clone(), walker);
        }
        self.sp_items.push(enum_item.clone());
        self.declarations
            .insert(enum_item.clone().read().unwrap().key(), enum_item);

        Ok(())
    }

    fn get_enum_name_and_range(
        &self,
        node: &Node,
        anon_enum_counter: &mut u32,
    ) -> Result<(String, Range), Utf8Error> {
        match node.child_by_field_name("name") {
            Some(name_node) => {
                let name = name_node.utf8_text(self.preprocessed_text.as_bytes())?;

                Ok((name.to_string(), ts_range_to_lsp_range(&name_node.range())))
            }
            None => {
                let mut name = String::from("Enum#");
                name.push_str(anon_enum_counter.to_string().as_str());
                let range = Range {
                    start: Position {
                        line: node.start_position().row as u32,
                        character: 0,
                    },
                    end: Position {
                        line: node.start_position().row as u32,
                        character: 0,
                    },
                };
                *anon_enum_counter += 1;

                Ok((name, range))
            }
        }
    }

    fn read_enum_members(
        &self,
        body_node: &Node,
        enum_item: Arc<RwLock<SPItem>>,
        walker: &mut Walker,
    ) {
        let mut cursor = body_node.walk();
        for child in body_node.children(&mut cursor) {
            match child.kind() {
                "enum_entry" => {
                    let _ = self.read_enum_entry(child, &enum_item);
                }
                "comment" => {
                    walker.push_comment(child, &self.preprocessed_text);
                    walker.push_inline_comment(enum_item.read().unwrap().children().unwrap());
                }
                "preproc_pragma" => {
                    let _ = walker.push_deprecated(child, &self.preprocessed_text);
                }
                _ => (),
            }
        }
    }

    fn read_enum_entry(&self, child: Node, enum_item: &Arc<RwLock<SPItem>>) -> anyhow::Result<()> {
        let name_node = child
            .child_by_field_name("name")
            .context("Enum entry has no name.")?;
        let name = name_node
            .utf8_text(self.preprocessed_text.as_bytes())?
            .to_string();
        let range = ts_range_to_lsp_range(&name_node.range());
        let enum_member_item = EnumMemberItem {
            name,
            uri: self.uri.clone(),
            range,
            v_range: self.build_v_range(&range),
            parent: Arc::downgrade(enum_item),
            description: Description::default(),
            references: vec![],
        };
        enum_item
            .write()
            .unwrap()
            .push_child(Arc::new(RwLock::new(SPItem::EnumMember(enum_member_item))));

        Ok(())
    }
}
