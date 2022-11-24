use std::collections::HashSet;

use lsp_types::{CompletionItem, CompletionParams, Url};

use crate::{document::Document, store::Store};

use self::function_item::FunctionItem;

pub mod function_item;

#[derive(Debug, Clone)]
/// Generic representation of an item, which can be converted to a
/// [CompletionItem](lsp_types::CompletionItem), [Location](lsp_types::Location), etc.
pub enum SPItem {
    Function(FunctionItem),
}

pub fn to_completion(sp_item: &SPItem, params: &CompletionParams) -> Option<CompletionItem> {
    match sp_item {
        SPItem::Function(function_item) => function_item::to_completion(function_item, params),
    }
}

pub fn get_all_items(store: &Store, main_path_uri: Url) -> Vec<SPItem> {
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
