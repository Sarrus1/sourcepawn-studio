use std::{
    str::Utf8Error,
    sync::{Arc, Mutex},
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
    let documentation = find_doc(walker, node.start_position().row, &document.text)?;

    let enum_item = EnumItem {
        name,
        range,
        full_range: ts_range_to_lsp_range(&node.range()),
        description: documentation,
        uri: document.uri.clone(),
        references: vec![],
    };

    let mut cursor = node.walk();
    let mut enum_entries: Option<Node> = None;
    for child in node.children(&mut cursor) {
        if child.kind() == "enum_entries" {
            enum_entries = Some(child);
            break;
        }
    }
    let enum_item = Arc::new(Mutex::new(SPItem::Enum(enum_item)));
    if enum_entries.is_some() {
        read_enum_members(
            document,
            &enum_entries.unwrap(),
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
    if name_node.is_some() {
        let name_node = name_node.unwrap();
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
    file_item: &mut Document,
    body_node: &Node,
    enum_item: Arc<Mutex<SPItem>>,
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
            parent: enum_item.clone(),
            description: Description::default(),
            references: vec![],
        };
        file_item
            .sp_items
            .push(Arc::new(Mutex::new(SPItem::EnumMember(enum_member_item))));
    }
}
