use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use lsp_types::Url;

pub mod analyzer;
pub mod inherit;
mod resolvers;
pub mod scope;

use crate::{
    document::Document,
    spitem::{get_all_items, SPItem},
    store::Store,
};

use self::{analyzer::Analyzer, resolvers::resolve_item};

impl Document {
    pub fn find_references(&self, store: &Store) {
        let all_items = get_all_items(store, false);
        let mut analyzer = Analyzer::new(all_items, self);
        for token in self.tokens.iter() {
            analyzer.update_scope(token.range);
            analyzer.update_line_context(token);
            resolve_item(&mut analyzer, token, self);

            analyzer.token_idx += 1;
        }
        resolve_methodmap_inherits(get_all_items(store, false));
    }
}

pub fn purge_references(item: &Arc<RwLock<SPItem>>, uri: &Arc<Url>) {
    let mut new_references = vec![];
    let mut item_lock = item.write().unwrap();
    let old_references = item_lock.references();
    if old_references.is_none() {
        return;
    }
    let old_references = old_references.unwrap();
    for reference in old_references {
        if reference.uri.ne(uri) {
            new_references.push(reference.clone());
        }
    }
    item_lock.set_new_references(new_references);
}

/// Resolve methodmap inheritances when possible.
///
/// # Arguments
///
/// * `all_items` - All included first level [items](SPItem).
pub fn resolve_methodmap_inherits(all_items: Vec<Arc<RwLock<SPItem>>>) {
    let mut methodmaps = HashMap::new();
    let mut methodmaps_to_resolve = vec![];
    all_items.iter().for_each(|item| {
        if let SPItem::Methodmap(mm_item) = &*item.read().unwrap() {
            methodmaps.insert(mm_item.name.to_string(), item.clone());
            if mm_item.tmp_parent.is_some() {
                methodmaps_to_resolve.push(item.clone());
            }
        }
    });

    for mm in methodmaps_to_resolve.iter() {
        let mut mm = mm.write().unwrap();
        if let SPItem::Methodmap(mm_item) = &*mm {
            if let Some(tmp_parent) = &mm_item.tmp_parent {
                if let Some(parent) = methodmaps.get(tmp_parent) {
                    mm.set_parent(parent.clone());
                }
            }
        }
    }
}
