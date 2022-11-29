use std::{str::Utf8Error, sync::Arc};

use tree_sitter::Node;

use crate::{
    document::{find_doc, Document},
    spitem::{enum_item::EnumItem, SPItem},
    utils::ts_range_to_lsp_range,
};

use lsp_types::{Position, Range};

pub fn parse_enum(
    document: &mut Document,
    node: &mut Node,
    comments: &mut Vec<Node>,
    deprecated: &mut Vec<Node>,
    anon_enum_counter: &mut u32,
) -> Result<(), Utf8Error> {
    let (name, range) = get_enum_name_and_range(node, &document.text, anon_enum_counter);
    let documentation = find_doc(
        comments,
        deprecated,
        node.start_position().row,
        &document.text,
    )?;

    let enum_item = EnumItem {
        name,
        range,
        full_range: ts_range_to_lsp_range(&node.range()),
        description: documentation,
        uri: document.uri.clone(),
        references: vec![],
    };
    let enum_item = Arc::new(SPItem::Enum(enum_item));
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
