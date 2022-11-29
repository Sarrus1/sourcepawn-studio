use std::{collections::HashSet, sync::Arc};

use lsp_types::{CompletionItem, CompletionParams, Location, Position, Range, Url};

use crate::{
    document::Document, providers::hover::description::Description, store::Store,
    utils::range_contains_pos,
};

pub mod enum_item;
pub mod enum_member_item;
pub mod function_item;
pub mod variable_item;

#[derive(Debug, Clone)]
/// Generic representation of an item, which can be converted to a
/// [CompletionItem](lsp_types::CompletionItem), [Location](lsp_types::Location), etc.
pub enum SPItem {
    Function(function_item::FunctionItem),
    Variable(variable_item::VariableItem),
    Enum(enum_item::EnumItem),
    EnumMember(enum_member_item::EnumMemberItem),
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

pub fn get_item_from_position(store: &Store, position: Position, uri: Url) -> Option<Arc<SPItem>> {
    let all_items = get_all_items(store);
    for item in all_items.iter() {
        match item.range() {
            Some(range) => {
                if range_contains_pos(range, position) && item.uri().as_ref().eq(&uri) {
                    return Some(item.clone());
                }
            }
            None => {}
        }
        match item.references() {
            Some(references) => {
                for reference in references.iter() {
                    if range_contains_pos(reference.range, position) && reference.uri.eq(&uri) {
                        return Some(item.clone());
                    }
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
            SPItem::Enum(item) => Some(item.range),
            SPItem::EnumMember(item) => Some(item.range),
        }
    }

    pub fn name(&self) -> String {
        match self {
            SPItem::Variable(item) => item.name.clone(),
            SPItem::Function(item) => item.name.clone(),
            SPItem::Enum(item) => item.name.clone(),
            SPItem::EnumMember(item) => item.name.clone(),
        }
    }

    pub fn description(&self) -> Option<Description> {
        match self {
            SPItem::Variable(item) => Some(item.description.clone()),
            SPItem::Function(item) => Some(item.description.clone()),
            SPItem::Enum(item) => Some(item.description.clone()),
            SPItem::EnumMember(item) => Some(item.description.clone()),
        }
    }

    pub fn uri(&self) -> Arc<Url> {
        match self {
            SPItem::Variable(item) => item.uri.clone(),
            SPItem::Function(item) => item.uri.clone(),
            SPItem::Enum(item) => item.uri.clone(),
            SPItem::EnumMember(item) => item.uri.clone(),
        }
    }

    pub fn references(&self) -> Option<&Vec<Location>> {
        match self {
            SPItem::Variable(item) => Some(&item.references),
            SPItem::Function(item) => Some(&item.references),
            SPItem::Enum(item) => Some(&item.references),
            SPItem::EnumMember(item) => Some(&item.references),
        }
    }

    pub fn to_completion(&self, params: &CompletionParams) -> Option<CompletionItem> {
        match self {
            SPItem::Variable(item) => item.to_completion(params),
            SPItem::Function(item) => item.to_completion(params),
            SPItem::Enum(item) => item.to_completion(params),
            SPItem::EnumMember(item) => item.to_completion(params),
        }
    }
}
