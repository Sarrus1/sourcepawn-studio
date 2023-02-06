use std::{
    str::Utf8Error,
    sync::{Arc, RwLock},
};

use tree_sitter::Node;

use crate::{
    document::{find_doc, Document, Walker},
    providers::hover::description::Description,
    spitem::{enum_item::EnumItem, enum_member_item::EnumMemberItem, SPItem},
    utils::ts_range_to_lsp_range,
};

use lsp_types::{Position, Range, Url};

pub fn parse_enum(
    document: &mut Document,
    node: &mut Node,
    walker: &mut Walker,
) -> Result<(), Utf8Error> {
    let (name, range) =
        get_enum_name_and_range(node, &document.text, &mut walker.anon_enum_counter);
    let documentation = find_doc(walker, node.start_position().row)?;

    let enum_item = EnumItem {
        name,
        range,
        full_range: ts_range_to_lsp_range(&node.range()),
        description: documentation,
        uri: document.uri.clone(),
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
        read_enum_members(
            &enum_entries,
            enum_item.clone(),
            &document.text.to_string(),
            document.uri.clone(),
        );
    }
    document.sp_items.push(enum_item);

    Ok(())
}

fn get_enum_name_and_range(
    node: &Node,
    source: &String,
    anon_enum_counter: &mut u32,
) -> (String, Range) {
    let name_node = node.child_by_field_name("name");
    if let Some(name_node) = name_node {
        let name_node = name_node;
        let name = name_node.utf8_text(source.as_bytes()).unwrap();
        return (name.to_string(), ts_range_to_lsp_range(&name_node.range()));
    }
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

    (name, range)
}

fn read_enum_members(
    body_node: &Node,
    enum_item: Arc<RwLock<SPItem>>,
    source: &String,
    uri: Arc<Url>,
) {
    let mut cursor = body_node.walk();
    for child in body_node.children(&mut cursor) {
        let kind = child.kind();
        if kind != "enum_entry" {
            continue;
        }
        let name_node = child.child_by_field_name("name").unwrap();
        let name = name_node.utf8_text(source.as_bytes()).unwrap().to_string();
        let range = ts_range_to_lsp_range(&name_node.range());
        let enum_member_item = EnumMemberItem {
            name,
            uri: uri.clone(),
            range,
            parent: Arc::downgrade(&enum_item),
            description: Description::default(),
            references: vec![],
        };
        enum_item
            .write()
            .unwrap()
            .push_child(Arc::new(RwLock::new(SPItem::EnumMember(enum_member_item))));
    }
}
