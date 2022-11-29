use std::{collections::HashSet, sync::Arc};

use lsp_types::{CompletionItem, CompletionParams, Position, Range, Url};

use crate::{document::Document, providers::hover::description::Description, store::Store};

use self::variable_item::range_contains_pos;

pub mod function_item;
pub mod variable_item;

#[derive(Debug, Clone)]
/// Generic representation of an item, which can be converted to a
/// [CompletionItem](lsp_types::CompletionItem), [Location](lsp_types::Location), etc.
pub enum SPItem {
    Function(function_item::FunctionItem),
    Variable(variable_item::VariableItem),
}

pub fn to_completion(sp_item: &SPItem, params: &CompletionParams) -> Option<CompletionItem> {
    match sp_item {
        SPItem::Variable(variable_item) => variable_item.to_completion(params),
        SPItem::Function(function_item) => function_item::to_completion(function_item, params),
    }
}

pub fn get_all_items(store: &Store) -> Vec<Arc<SPItem>> {
    let main_path = store.environment.options.main_path.clone();
    let main_path_uri = Url::from_file_path(main_path).expect("Invalid main path");
    let mut includes: HashSet<Url> = HashSet::new();
    includes.insert(main_path_uri.clone());
    let mut all_items = vec![];
    if let Some(document) = store.get(&main_path_uri) {
        get_included_files(store, document, &mut includes);
        for include in includes.iter() {
            let document = store.get(include).unwrap();
            all_items.extend(document.sp_items);
        }
    }

    all_items
}

fn get_included_files(store: &Store, document: Document, includes: &mut HashSet<Url>) {
    for include_uri in document.includes.iter() {
        if includes.contains(include_uri) {
            return;
        }
        includes.insert(include_uri.clone());
        if let Some(include_document) = store.get(include_uri) {
            get_included_files(store, include_document, includes);
        }
    }
}

pub fn get_item_from_position(store: &Store, pos: Position) -> Option<Arc<SPItem>> {
    let all_items = get_all_items(store);
    for item in all_items.iter() {
        match item.range() {
            Some(range) => {
                if range_contains_pos(range, pos) {
                    return Some(item.clone());
                }
            }
            None => {
                continue;
            }
        }
    }

    None
}

impl SPItem {
    pub fn range(&self) -> Option<Range> {
        match self {
            SPItem::Variable(item) => Some(item.range),
            SPItem::Function(item) => Some(item.range),
        }
    }

    pub fn name(&self) -> Option<String> {
        match self {
            SPItem::Variable(item) => Some(item.name.clone()),
            SPItem::Function(item) => Some(item.name.clone()),
        }
    }

    pub fn description(&self) -> Option<Description> {
        match self {
            SPItem::Variable(item) => Some(item.description.clone()),
            SPItem::Function(item) => Some(item.description.clone()),
        }
    }
}
