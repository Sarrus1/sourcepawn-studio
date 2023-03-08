use std::sync::{Arc, RwLock};

use crate::{
    document::{Document, Token},
    providers::completion::context::is_ctr_call,
    spitem::{Location, SPItem},
    utils::range_contains_range,
};

use super::{analyzer::Analyzer, inherit::find_inherit};

impl Analyzer {
    /// Try to solve for the `this` token. Returns `false` only if the token's text is not
    /// `this`. Otherwise, will return `true` when it matches of when it ends.
    ///
    /// # Arguments
    ///
    /// * `token` - [Token] to analyze.
    /// * `document` - [Document](super::document::Document) to analyze.
    fn resolve_this(&mut self, token: &Arc<Token>, document: &Document) -> bool {
        if token.text != "this" {
            return false;
        }
        for item in self.all_items.iter() {
            let item_lock = item.read().unwrap();
            match &*item_lock {
                SPItem::Methodmap(mm_item) => {
                    if mm_item.uri.eq(&document.uri)
                        && range_contains_range(&mm_item.full_range, &token.range)
                    {
                        self.previous_items.push(item.clone());
                        return true;
                    }
                }
                SPItem::EnumStruct(es_item) => {
                    if es_item.uri.eq(&document.uri)
                        && range_contains_range(&es_item.full_range, &token.range)
                    {
                        self.previous_items.push(item.clone());
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
    /// * `token` - [Token] to analyze.
    /// * `document` - [Document](super::document::Document) to analyze.
    fn resolve_non_method_item(&mut self, token: &Arc<Token>, document: &Document) -> bool {
        let full_key = format!(
            "{}-{}-{}",
            self.scope.mm_es_key(),
            self.scope.func_key(),
            token.text
        );
        let semi_key = format!("{}-{}", self.scope.mm_es_key(), token.text);
        let mid_key = format!("{}-{}", self.scope.func_key(), token.text);

        let item = self
            .tokens_map
            .get(&full_key)
            .or_else(|| self.tokens_map.get(&mid_key))
            .or_else(|| self.tokens_map.get(&semi_key))
            .or_else(|| self.tokens_map.get(&token.text));

        if let Some(item) = item {
            let reference = Location {
                uri: document.uri.clone(),
                range: token.range,
            };

            if let SPItem::Methodmap(mm_item) = &*item.read().unwrap() {
                if token.range.start.character >= 4 {
                    // Don't check the line if there is not enough space for a `new` keyword.
                    // We use 4 instead of 3 to account for at least one space after `new`.
                    let pre_line: String = self
                        .line()
                        .chars()
                        .take(token.range.start.character as usize)
                        .collect();
                    if is_ctr_call(&pre_line) {
                        if let Some(ctr_item) = mm_item.ctr() {
                            ctr_item.write().unwrap().push_reference(reference);
                            self.previous_items.push(ctr_item);
                            return true;
                        }
                    }
                }
            }

            item.write().unwrap().push_reference(reference);
            self.previous_items.push(item.clone());
            return true;
        }

        false
    }

    pub(super) fn resolve_item(&mut self, token: &Arc<Token>, document: &Document) -> Option<()> {
        if self.resolve_this(token, document) {
            return Some(());
        }

        if self.resolve_non_method_item(token, document) {
            return Some(());
        }

        if token.range.start.character > 0 && !self.previous_items.is_empty() {
            let char = self
                .line()
                .chars()
                .nth((token.range.start.character - 1) as usize)
                .unwrap();
            if char != ':' && char != '.' {
                return None;
            }
            let mut item: Option<Arc<RwLock<SPItem>>> = None;
            for parent in self.previous_items.iter().rev() {
                let parent = parent.read().unwrap().clone();
                match &parent {
                    SPItem::EnumStruct(es) => {
                        // Enum struct scope operator (::).
                        item = self.get(&format!("{}-{}", es.name, token.text));
                        if item.is_some() {
                            break;
                        }
                    }
                    SPItem::Methodmap(mm) => {
                        // Methodmap static method.
                        item = self.get(&format!("{}-{}", mm.name, token.text));
                        if item.is_some() {
                            break;
                        }
                    }
                    _ => {}
                }
                item = self.get(&format!("{}-{}", parent.type_(), token.text));
                if item.is_some() {
                    break;
                }
                for inherit in find_inherit(&self.all_items, &parent) {
                    item = self.get(&format!(
                        "{}-{}",
                        inherit.read().unwrap().name(),
                        token.text
                    ));
                    if item.is_some() {
                        break;
                    }
                }
            }
            item.as_ref()?;
            let item = item.unwrap();
            let reference = Location {
                uri: document.uri.clone(),
                range: token.range,
            };
            item.write().unwrap().push_reference(reference);
            self.previous_items.push(item);

            return Some(());
        }

        None
        // TODO: Handle positional arguments
    }
}
