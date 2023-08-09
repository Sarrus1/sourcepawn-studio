use lazy_static::lazy_static;
use parking_lot::RwLock;
use regex::Regex;
use std::sync::Arc;
use syntax::{Location, SPItem};

use crate::{
    document::{Document, Token},
    range_contains_range,
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
    pub(super) fn resolve_this(&mut self, token: &Arc<Token>, document: &Document) -> bool {
        if token.text != "this" {
            return false;
        }
        for item in self.all_items.iter() {
            let item_lock = item.read();
            match &*item_lock {
                SPItem::Methodmap(mm_item) => {
                    if mm_item.uri.eq(&document.uri)
                        && range_contains_range(&mm_item.full_range, &token.range)
                    {
                        self.previous_items.insert(token.text.clone(), item.clone());
                        return true;
                    }
                }
                SPItem::EnumStruct(es_item) => {
                    if es_item.uri.eq(&document.uri)
                        && range_contains_range(&es_item.full_range, &token.range)
                    {
                        self.previous_items.insert(token.text.clone(), item.clone());
                        return true;
                    }
                }
                _ => {
                    continue;
                }
            }
        }

        // TODO: this keyword outside of its scope but we ignore it for now.
        true
    }

    /// Try to solve for a non method token, i.e which does not depend on the type of the previous
    /// token on the same line. Returns `true` if it did resolve, `false` otherwise.
    ///
    /// # Arguments
    ///
    /// * `token` - [Token] to analyze.
    /// * `document` - [Document](super::document::Document) to analyze.
    pub(super) fn resolve_non_method_item(
        &mut self,
        token: &Arc<Token>,
        document: &Document,
    ) -> anyhow::Result<()> {
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
                v_range: if let SPItem::Define(_) = &*item.read() {
                    token.range
                } else {
                    document.build_v_range(&token.range)
                },
            };

            if let SPItem::Methodmap(mm_item) = &*item.read() {
                if token.range.start.character >= 4 {
                    // Don't check the line if there is not enough space for a `new` keyword.
                    // We use 4 instead of 3 to account for at least one space after `new`.
                    let pre_line: String = self
                        .line()?
                        .chars()
                        .take(token.range.start.character as usize)
                        .collect();
                    if is_ctor_call(&pre_line) {
                        if let Some(ctor_item) = mm_item.ctor() {
                            ctor_item.write().push_reference(reference);
                            self.previous_items.insert(token.text.clone(), ctor_item);
                            return Ok(());
                        }
                    }
                }
            }

            item.write().push_reference(reference);
            self.previous_items.insert(token.text.clone(), item.clone());
            return Ok(());
        }

        anyhow::bail!("Token not found.");
    }

    pub(super) fn resolve_method_item(
        &mut self,
        parent: &Arc<Token>,
        field: &Arc<Token>,
        document: &Document,
    ) -> Option<()> {
        if self.previous_items.is_empty() {
            return None;
        }

        let mut item: Option<Arc<RwLock<SPItem>>> = None;
        let parent_item = self.previous_items.get(parent.text.as_str())?;
        let parent = parent_item.read().clone();
        match &parent {
            SPItem::EnumStruct(es) => {
                // Enum struct scope operator (::).
                item = self.get(&format!("{}-{}", es.name, field.text));
            }
            SPItem::Methodmap(mm) => {
                // Methodmap static method.
                item = self.get(&format!("{}-{}", mm.name, field.text));
            }
            _ => {}
        }
        if item.is_none() {
            item = self.get(&format!("{}-{}", parent.type_(), field.text));
        }
        if item.is_none() {
            for inherit in find_inherit(&self.all_items, &parent) {
                item = self.get(&format!("{}-{}", inherit.read().name(), field.text));
                if item.is_some() {
                    break;
                }
            }
        }

        item.as_ref()?;
        let item = item.unwrap();
        let reference = Location {
            uri: document.uri.clone(),
            range: field.range,
            v_range: document.build_v_range(&field.range),
        };
        item.write().push_reference(reference);
        self.previous_items.insert(field.text.clone(), item);

        // TODO: Handle positional arguments
        Some(())
    }
}

/// Given a prefix line of a document, return whether or not the end of the prefix line is right after
/// a constructor call i.e after a `new`.
///
/// # Arguments
///
/// * `pre_line` - Prefix line to check against.
pub fn is_ctor_call(pre_line: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"new\s+\w*$").unwrap();
    }
    RE.is_match(pre_line)
}
