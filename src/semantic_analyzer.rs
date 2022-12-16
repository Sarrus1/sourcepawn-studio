use std::sync::{Arc, Mutex};

use lsp_types::Url;

pub mod analyzer;
pub mod inherit;
pub mod scope;

use crate::{
    document::{Document, Token},
    spitem::{get_all_items, Location, SPItem},
    store::Store,
};

use self::{analyzer::Analyzer, inherit::find_inherit};

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
            resolve_item(&mut analyzer, token, self);

            analyzer.token_idx += 1;
        }
    }
}

fn resolve_item(analyzer: &mut Analyzer, token: &Arc<Token>, document: &Document) {
    if token.range.start.line != analyzer.line_nb || analyzer.token_idx == 0 {
        analyzer.line_nb = token.range.start.line;
        analyzer.previous_items.clear();
    }
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
