use std::sync::{Arc, RwLock};

use fxhash::{FxHashMap, FxHashSet};
use lsp_types::Url;

pub mod analyzer;
pub mod inherit;
mod resolvers;
pub mod scope;

use crate::{
    spitem::{get_all_items, SPItem},
    store::Store,
};

use self::{analyzer::Analyzer, resolvers::resolve_item};

impl Store {
    pub fn find_references(&mut self, uri: &Url) {
        if !self.documents.contains_key(uri) {
            return;
        }
        let all_items = get_all_items(self, false);
        let document = self.documents.get(uri).unwrap();
        let mut unresolved_tokens = FxHashSet::default();
        let mut analyzer = Analyzer::new(all_items, document);
        for token in document.tokens.iter() {
            analyzer.update_scope(token.range);
            analyzer.update_line_context(token);
            if resolve_item(&mut analyzer, token, document).is_none() {
                // Token was not resolved
                unresolved_tokens.insert(token.text.clone());
            }

            analyzer.token_idx += 1;
        }
        resolve_methodmap_inherits(get_all_items(self, false));
        let document = self.documents.get_mut(uri).unwrap();
        document.unresolved_tokens = unresolved_tokens;
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
    let mut methodmaps = FxHashMap::default();
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
