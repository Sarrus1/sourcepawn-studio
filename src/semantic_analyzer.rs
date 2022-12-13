use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use lazy_static::lazy_static;
use lsp_types::Url;
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
    utils::{range_contains_range, ts_range_to_lsp_range},
};

pub fn find_references(store: &Store, root_node: Node, document: Document) {
    let all_items = get_all_items(store);
    if all_items.is_none() {
        return;
    }
    let all_items = all_items.unwrap();
    let (tokens_maps, funcs_and_methods_in_file) = build_tokens_map(all_items, &document.uri);

    // let mut lines = document.text.lines();
    let mut func_scope: Option<Arc<Mutex<SPItem>>> = None;
    // let mut es_ms_scope: Option<Arc<Mutex<SPItem>>> = None;
    // let line = lines.next();
    // let mut lineNb = 0;
    // let mut scope = "";
    // let mut outsideScope = "";
    // this.lastMMorES = undefined;
    // this.inTypeDef = false;

    let mut func_idx = 0;
    // let mut es_ms_idx = 0;
    // let mut typeIdx = 0;
    let mut cursor = QueryCursor::new();
    let matches = cursor.captures(&SYMBOL_QUERY, root_node, document.text.as_bytes());
    for (match_, _) in matches {
        for capture in match_.captures.iter() {
            let text = capture
                .node
                .utf8_text(document.text.as_bytes())
                .unwrap()
                .to_string();
            let range = ts_range_to_lsp_range(&capture.node.range());

            if func_scope.is_none()
                || !range_contains_range(
                    &range,
                    &func_scope
                        .as_ref()
                        .unwrap()
                        .lock()
                        .unwrap()
                        .full_range()
                        .unwrap(),
                )
            {
                if func_idx < funcs_and_methods_in_file.len()
                    && range_contains_range(
                        &range,
                        &funcs_and_methods_in_file[func_idx]
                            .lock()
                            .unwrap()
                            .full_range()
                            .unwrap(),
                    )
                {
                    func_scope = Some(funcs_and_methods_in_file[func_idx].clone());
                    func_idx += 1;
                } else {
                    func_scope = None;
                }
            }
            let key: String;
            if func_scope.is_some() {
                key = format!(
                    "{}-{}",
                    func_scope.clone().unwrap().lock().unwrap().name(),
                    text
                );
            } else {
                key = text.clone();
            }

            let item = tokens_maps.get(&key).or_else(|| tokens_maps.get(&text));
            if item.is_some() {
                let item = item.unwrap();
                let reference = Location {
                    uri: document.uri.clone(),
                    range: ts_range_to_lsp_range(&capture.node.range()),
                };
                item.lock().unwrap().push_reference(reference);
                continue;
            }
        }
    }
}

/// key format: "{outermost_scope}-{outer_scope}-{item_name}"
/// key format: "{outer_scope}-{item_name}"
fn build_tokens_map(
    all_items: Vec<Arc<Mutex<SPItem>>>,
    uri: &Arc<Url>,
) -> (HashMap<String, Arc<Mutex<SPItem>>>, Vec<Arc<Mutex<SPItem>>>) {
    let mut tokens_map: HashMap<String, Arc<Mutex<SPItem>>> = HashMap::new();
    let mut funcs_and_methods_in_file = vec![];

    for item in all_items.iter() {
        purge_references(item, &uri);
        match &*item.lock().unwrap() {
            // Match variables
            SPItem::Variable(variable_item) => match &variable_item.parent {
                // Match non global variables
                Some(variable_item_parent) => match &*variable_item_parent.lock().unwrap() {
                    // Match variables in a function or method
                    SPItem::Function(variable_item_parent_function) => {
                        let key = format!(
                            "{}-{}",
                            variable_item_parent_function.name, variable_item.name
                        );
                        tokens_map.insert(key, item.clone());
                    }
                    // Match variables as enum struct fields
                    SPItem::EnumStruct(variable_item_parent_enum_struct) => {
                        let key = format!(
                            "{}-{}",
                            variable_item_parent_enum_struct.name, variable_item.name
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
                    if function_item.uri.eq(uri) {
                        funcs_and_methods_in_file.push(item.clone());
                    }
                    tokens_map.insert(function_item.name.to_string(), item.clone());
                }
            },
            SPItem::Methodmap(methodmap_item) => {
                tokens_map.insert(methodmap_item.name.to_string(), item.clone());
            }
            SPItem::EnumStruct(enum_struct_item) => {
                tokens_map.insert(enum_struct_item.name.to_string(), item.clone());
            }
            SPItem::Enum(enum_item) => {
                tokens_map.insert(enum_item.name.to_string(), item.clone());
            }
            SPItem::Define(define_item) => {
                tokens_map.insert(define_item.name.to_string(), item.clone());
            }
            SPItem::EnumMember(enum_member_item) => {
                tokens_map.insert(enum_member_item.name.to_string(), item.clone());
            } // TODO: add typedef and typeset here
            _ => {}
        }
    }

    (tokens_map, funcs_and_methods_in_file)
}

fn purge_references(item: &Arc<Mutex<SPItem>>, uri: &Arc<Url>) {
    let mut new_references = vec![];
    let item_lock = item.lock().unwrap();
    let old_references = item_lock.references();
    if old_references.is_none() {
        return;
    }
    let old_references = old_references.unwrap();
    for reference in old_references {
        if reference.uri.eq(&uri) {
            new_references.push(reference);
        }
    }
}
