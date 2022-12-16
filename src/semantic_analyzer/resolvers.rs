use std::sync::{Arc, Mutex};

use crate::{
    document::{Document, Token},
    spitem::{Location, SPItem},
    utils::range_contains_range,
};

use super::{analyzer::Analyzer, inherit::find_inherit};

/// Try to solve for the `this` token. Returns `false` only if the token's text is not
/// `this`. Otherwise, will return `true` when it matches of when it ends.
///
/// # Arguments
///
/// * `analyzer` - [Analyzer] object.
/// * `token` - [Token] to analyze.
/// * `document` - [Document](super::document::Document) to analyze.
fn resolve_this(analyzer: &mut Analyzer, token: &Arc<Token>, document: &Document) -> bool {
    if token.text != "this" {
        return false;
    }
    for item in analyzer.all_items.iter() {
        let item_lock = item.lock().unwrap();
        match &*item_lock {
            SPItem::Methodmap(mm_item) => {
                if mm_item.uri.eq(&document.uri)
                    && range_contains_range(&mm_item.full_range, &token.range)
                {
                    analyzer.previous_items.push(item.clone());
                    return true;
                }
            }
            SPItem::EnumStruct(es_item) => {
                if es_item.uri.eq(&document.uri)
                    && range_contains_range(&es_item.full_range, &token.range)
                {
                    analyzer.previous_items.push(item.clone());
                    return true;
                }
            }
            _ => {
                continue;
            }
        }
    }

    true
}

/// Try to solve for a non method token, i.e which does not depend on the type of the previous
/// token on the same line. Returns `true` if it did resolve, `false` otherwise.
///
/// # Arguments
///
/// * `analyzer` - [Analyzer] object.
/// * `token` - [Token] to analyze.
/// * `document` - [Document](super::document::Document) to analyze.
fn resolve_non_method_item(
    analyzer: &mut Analyzer,
    token: &Arc<Token>,
    document: &Document,
) -> bool {
    let full_key = format!(
        "{}-{}-{}",
        analyzer.scope.mm_es_key(),
        analyzer.scope.func_key(),
        token.text
    );
    let semi_key = format!("{}-{}", analyzer.scope.mm_es_key(), token.text);
    let mid_key = format!("{}-{}", analyzer.scope.func_key(), token.text);

    let item = analyzer
        .tokens_map
        .get(&full_key)
        .or_else(|| analyzer.tokens_map.get(&mid_key))
        .or_else(|| analyzer.tokens_map.get(&semi_key))
        .or_else(|| analyzer.tokens_map.get(&token.text));

    if let Some(item) = item {
        let item = item;
        let reference = Location {
            uri: document.uri.clone(),
            range: token.range,
        };
        item.lock().unwrap().push_reference(reference);
        analyzer.previous_items.push(item.clone());
        return true;
    }

    false
}

pub(super) fn resolve_item(analyzer: &mut Analyzer, token: &Arc<Token>, document: &Document) {
    if resolve_this(analyzer, token, document) {
        return;
    }

    if resolve_non_method_item(analyzer, token, document) {
        return;
    }

    if token.range.start.character > 0 && !analyzer.previous_items.is_empty() {
        let char = analyzer.line().as_bytes()[(token.range.start.character - 1) as usize] as char;
        if char != ':' && char != '.' {
            return;
        }
        let mut item: Option<Arc<Mutex<SPItem>>> = None;
        for parent in analyzer.previous_items.iter().rev() {
            let parent = parent.lock().unwrap().clone();
            match &parent {
                SPItem::EnumStruct(es) => {
                    // Enum struct scope operator (::).
                    item = analyzer.get(&format!("{}-{}", es.name, token.text));
                    if item.is_some() {
                        break;
                    }
                }
                SPItem::Methodmap(mm) => {
                    // Methodmap static method.
                    item = analyzer.get(&format!("{}-{}", mm.name, token.text));
                    if item.is_some() {
                        break;
                    }
                }
                _ => {}
            }
            item = analyzer.get(&format!("{}-{}", parent.type_(), token.text));
            if item.is_some() {
                break;
            }
            for inherit in find_inherit(&analyzer.all_items, &parent) {
                item = analyzer.get(&format!(
                    "{}-{}",
                    inherit.lock().unwrap().name(),
                    token.text
                ));
                if item.is_some() {
                    break;
                }
            }
        }
        if item.is_none() {
            return;
        }
        let item = item.unwrap();
        let reference = Location {
            uri: document.uri.clone(),
            range: token.range,
        };
        item.lock().unwrap().push_reference(reference);
        analyzer.previous_items.push(item);
    }
    // TODO: Handle positional arguments
}
