use anyhow::Context;
use lsp_types::{Position, Range};
use parking_lot::RwLock;
use std::{str::Utf8Error, sync::Arc};
use syntax::{
    description::Description, enum_item::EnumItem, enum_member_item::EnumMemberItem,
    utils::ts_range_to_lsp_range, SPItem,
};
use tree_sitter::Node;

use crate::Parser;

impl<'a> Parser<'a> {
    pub fn parse_enum(&mut self, node: &mut Node) -> anyhow::Result<()> {
        let (name, range) = self.get_enum_name_and_range(node)?;
        let description = self
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
            file_id: self.file_id,
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
            self.read_enum_members(&enum_entries, enum_item.clone());
        }
        self.sp_items.push(enum_item.clone());
        self.declarations
            .insert(enum_item.clone().read().key(), enum_item);

        Ok(())
    }

    fn get_enum_name_and_range(&mut self, node: &Node) -> Result<(String, Range), Utf8Error> {
        match node.child_by_field_name("name") {
            Some(name_node) => {
                let name = name_node.utf8_text(self.source.as_bytes())?;

                Ok((name.to_string(), ts_range_to_lsp_range(&name_node.range())))
            }
            None => {
                let mut name = String::from("Enum#");
                name.push_str(self.anon_enum_counter.to_string().as_str());
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
                self.anon_enum_counter += 1;

                Ok((name, range))
            }
        }
    }

    fn read_enum_members(&mut self, body_node: &Node, enum_item: Arc<RwLock<SPItem>>) {
        let mut cursor = body_node.walk();
        for child in body_node.children(&mut cursor) {
            match child.kind() {
                "enum_entry" => {
                    let _ = self.read_enum_entry(child, &enum_item);
                }
                "comment" => {
                    self.push_comment(child);
                    if let Some(items) = enum_item.read().children() {
                        let Some(item) = items.last()else{continue;};
                        self.push_inline_comment(item);
                    }
                }
                "preproc_pragma" => {
                    let _ = self.push_deprecated(child);
                }
                _ => (),
            }
        }
    }

    fn read_enum_entry(&self, child: Node, enum_item: &Arc<RwLock<SPItem>>) -> anyhow::Result<()> {
        let name_node = child
            .child_by_field_name("name")
            .context("Enum entry has no name.")?;
        let name = name_node.utf8_text(self.source.as_bytes())?.to_string();
        let range = ts_range_to_lsp_range(&name_node.range());
        let enum_member_item = EnumMemberItem {
            name,
            uri: self.uri.clone(),
            file_id: self.file_id,
            range,
            v_range: self.build_v_range(&range),
            parent: Arc::downgrade(enum_item),
            description: Description::default(),
            references: vec![],
        };
        enum_item
            .write()
            .push_child(Arc::new(RwLock::new(SPItem::EnumMember(enum_member_item))));

        Ok(())
    }
}
