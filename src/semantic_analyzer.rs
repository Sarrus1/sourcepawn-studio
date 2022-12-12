use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use lazy_static::lazy_static;
use tree_sitter::{Node, Query, QueryCursor};

lazy_static! {
    static ref SYMBOL_QUERY: Query = {
        Query::new(
            tree_sitter_sourcepawn::language(),
            "[(symbol) @symbol (this) @symbol]",
        )
        .unwrap()
    };
}
use crate::{
    document::Document,
    spitem::{get_all_items, Location, SPItem},
    store::Store,
    utils::ts_range_to_lsp_range,
};

pub fn find_references(store: &Store, root_node: Node, document: Document) {
    let all_items = get_all_items(store);
    if all_items.is_none() {
        return;
    }
    let all_items = all_items.unwrap();
    let tokens_maps = build_tokens_map(all_items);
    let mut cursor = QueryCursor::new();
    let matches = cursor.captures(&SYMBOL_QUERY, root_node, document.text.as_bytes());
    for (match_, _) in matches {
        for capture in match_.captures.iter() {
            let text = capture.node.utf8_text(document.text.as_bytes()).unwrap();
            let item = tokens_maps.get(&text.to_string());
            match item {
                Some(item) => {
                    let reference = Location {
                        uri: document.uri.clone(),
                        range: ts_range_to_lsp_range(&capture.node.range()),
                    };
                    item.lock().unwrap().push_reference(reference);
                }
                None => {
                    continue;
                }
            }
        }
    }
}

fn build_tokens_map(all_items: Vec<Arc<Mutex<SPItem>>>) -> HashMap<String, Arc<Mutex<SPItem>>> {
    let mut tokens_map: HashMap<String, Arc<Mutex<SPItem>>> = HashMap::new();

    for item in all_items.iter() {
        match &*item.lock().unwrap() {
            SPItem::Variable(variable_item) => match &variable_item.parent {
                Some(variable_item_parent) => match &*variable_item_parent.lock().unwrap() {
                    SPItem::Function(variable_item_parent_function) => {
                        let key = format!(
                            "{}-{}",
                            variable_item_parent_function.name, variable_item.name
                        );
                        tokens_map.insert(key, item.clone());
                    }
                    _ => {}
                },
                None => {
                    tokens_map.insert(variable_item.name.to_string(), item.clone());
                }
            },
            SPItem::Function(function_item) => match &function_item.parent {
                Some(method_item_parent) => match &*method_item_parent.lock().unwrap() {
                    SPItem::Methodmap(method_item_parent) => {
                        let key = format!("{}-{}", method_item_parent.name, function_item.name);
                        tokens_map.insert(key, item.clone());
                    }
                    SPItem::EnumStruct(method_item_parent) => {
                        let key = format!("{}-{}", method_item_parent.name, function_item.name);
                        tokens_map.insert(key, item.clone());
                    }
                    _ => {}
                },
                None => {
                    tokens_map.insert(function_item.name.to_string(), item.clone());
                }
            },
            _ => {}
        }
    }

    tokens_map
}
