use std::sync::{Arc, Mutex};

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
        let all_items = get_all_items(store);
        if all_items.is_none() {
            return;
        }
        let all_items = all_items.unwrap();
        let mut analyzer = Analyzer::new(all_items, self);
        for token in self.tokens.iter() {
            analyzer.update_scope(token.range);
            analyzer.update_line_context(token);
            resolve_item(&mut analyzer, token, self);

            analyzer.token_idx += 1;
        }
    }
}

fn purge_references(item: &Arc<Mutex<SPItem>>, uri: &Arc<Url>) {
    let mut new_references = vec![];
    let mut item_lock = item.lock().unwrap();
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
