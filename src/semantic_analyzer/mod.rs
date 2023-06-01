use std::sync::{Arc, RwLock};

use fxhash::{FxHashMap, FxHashSet};
use lsp_types::Url;

pub mod analyzer;
pub mod inherit;
mod resolvers;
pub mod scope;

use crate::{document::SPToken, spitem::SPItem, store::Store};

use self::analyzer::Analyzer;

impl Store {
    pub(crate) fn find_references(&mut self, uri: &Url) -> Option<()> {
        log::trace!("Resolving references for document {:?}", uri);
        if !self.documents.contains_key(uri) {
            log::trace!("Skipped resolving references for document {:?}", uri);
            return None;
        }
        let all_items = self.get_all_items(false);
        let document = self.documents.get_mut(uri)?;
        let mut unresolved_tokens = FxHashSet::default();
        let mut analyzer = Analyzer::new(all_items, document);
        document.tokens.sort_by_key(|sp_token| match sp_token {
            SPToken::Symbol(token) => token.range.start.line,
            SPToken::Method((_, field)) => field.range.start.line,
        });
        for token in document.tokens.iter() {
            match token {
                SPToken::Symbol(token) => {
                    analyzer.update_scope(token.range);
                    analyzer.update_line_context(token);
                    if analyzer.resolve_this(token, document) {
                        analyzer.token_idx += 1;
                        continue;
                    }
                    if analyzer.resolve_non_method_item(token, document).is_ok() {
                        analyzer.token_idx += 1;
                        continue;
                    }
                    // Token was not resolved
                    unresolved_tokens.insert(token.text.clone());
                }
                SPToken::Method((parent, field)) => {
                    analyzer.update_scope(parent.range);
                    analyzer.update_line_context(parent);
                    if analyzer
                        .resolve_method_item(parent, field, document)
                        .is_none()
                    {
                        // Token was not resolved
                        unresolved_tokens.insert(field.text.clone());
                    }
                    analyzer.token_idx += 1;
                }
            }
        }
        resolve_methodmap_inherits(analyzer.all_items);
        let document = self.documents.get_mut(uri).unwrap();
        document.unresolved_tokens = unresolved_tokens;
        document.offsets.clear();
        log::trace!("Done resolving references for document {:?}", uri);

        Some(())
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
