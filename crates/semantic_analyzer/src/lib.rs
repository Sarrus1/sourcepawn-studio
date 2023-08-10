use fxhash::{FxHashMap, FxHashSet};
use lsp_types::{Range, Url};
use parking_lot::RwLock;
use preprocessor::Offset;
use std::sync::Arc;
use syntax::SPItem;

mod analyzer;
mod inherit;
mod resolvers;
mod scope;
mod token;

pub use {resolvers::is_ctor_call, token::*};

use crate::analyzer::Analyzer;

pub fn find_references(
    all_items: Vec<Arc<RwLock<SPItem>>>,
    uri: &Arc<Url>,
    source: &str,
    tokens: &mut [SPToken],
    offsets: &mut FxHashMap<u32, Vec<Offset>>,
) -> Option<FxHashSet<String>> {
    let mut unresolved_tokens = FxHashSet::default();
    let mut analyzer = Analyzer::new(all_items, uri.clone(), source, offsets);
    tokens.sort_by_key(|sp_token| match sp_token {
        SPToken::Symbol(token) => token.range.start.line,
        SPToken::Method((_, field)) => field.range.start.line,
    });
    for token in tokens.iter() {
        match token {
            SPToken::Symbol(token) => {
                analyzer.update_scope(token.range);
                analyzer.update_line_context(token);
                if analyzer.resolve_this(token, uri) {
                    analyzer.token_idx += 1;
                    continue;
                }
                if analyzer.resolve_non_method_item(token, uri).is_ok() {
                    analyzer.token_idx += 1;
                    continue;
                }
                // Token was not resolved
                unresolved_tokens.insert(token.text.clone());
            }
            SPToken::Method((parent, field)) => {
                analyzer.update_scope(parent.range);
                analyzer.update_line_context(parent);
                if analyzer.resolve_method_item(parent, field, uri).is_none() {
                    // Token was not resolved
                    unresolved_tokens.insert(field.text.clone());
                }
                analyzer.token_idx += 1;
            }
        }
    }
    resolve_methodmap_inherits(analyzer.all_items);
    offsets.clear();
    log::trace!("Done resolving references for document {:?}", uri);

    Some(unresolved_tokens)
}

pub fn purge_references(item: &Arc<RwLock<SPItem>>, uri: &Arc<Url>) {
    let mut new_references = vec![];
    let mut item_lock = item.write();
    let old_references = item_lock.references();
    if old_references.is_none() {
        return;
    }
    let old_references = old_references.unwrap();
    for reference in old_references {
        if reference.uri != *uri {
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
        if let SPItem::Methodmap(mm_item) = &*item.read() {
            methodmaps.insert(mm_item.name.to_string(), item.clone());
            if mm_item.tmp_parent.is_some() {
                methodmaps_to_resolve.push(item.clone());
            }
        }
    });

    for mm in methodmaps_to_resolve.iter() {
        let mut mm = mm.write();
        if let SPItem::Methodmap(mm_item) = &*mm {
            if let Some(tmp_parent) = &mm_item.tmp_parent {
                if let Some(parent) = methodmaps.get(tmp_parent) {
                    mm.set_parent(parent.clone());
                }
            }
        }
    }
}

/// Returns true if [Range] a contains [Range] b.
///
/// # Arguments
///
/// * `a` - [Range] to check against.
/// * `b` - [Range] to check against.
pub fn range_contains_range(a: &Range, b: &Range) -> bool {
    if b.start.line < a.start.line || b.end.line > a.end.line {
        return false;
    }
    if b.start.line == a.start.line && b.start.character < a.start.character {
        return false;
    }
    if b.end.line == a.end.line && b.end.character > a.end.character {
        return false;
    }

    true
}
