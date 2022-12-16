use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use lsp_types::{Range, Url};
use tree_sitter::{Node, Query, QueryCapture, QueryCursor};

pub mod analyzer;
pub mod inherit;
pub mod scope;

use crate::{
    document::Document,
    spitem::{get_all_items, Location, SPItem},
    store::Store,
    utils::ts_range_to_lsp_range,
};

use self::{analyzer::Analyzer, inherit::find_inherit};

lazy_static! {
    static ref SYMBOL_QUERY: Query = {
        Query::new(
            tree_sitter_sourcepawn::language(),
            "[(symbol) @symbol (this) @symbol]",
        )
        .unwrap()
    };
}

fn capture_text_range(capture: &QueryCapture, source: &String) -> (String, Range) {
    let text = capture
        .node
        .utf8_text(source.as_bytes())
        .unwrap()
        .to_string();
    let range = ts_range_to_lsp_range(&capture.node.range());

    (text, range)
}

impl Document {
    pub fn find_references(&self, store: &Store, root_node: Node) {
        let all_items = get_all_items(store);
        if all_items.is_none() {
            return;
        }
        let all_items = all_items.unwrap();
        let mut analyzer = Analyzer::new(all_items, &self);
        let mut cursor = QueryCursor::new();
        let matches = cursor.captures(&SYMBOL_QUERY, root_node, self.text.as_bytes());
        for (match_, _) in matches {
            for capture in match_.captures.iter() {
                let (token, range) = capture_text_range(capture, &self.text);

                analyzer.update_scope(range);
                resolve_item(&mut analyzer, &token, range, &self);

                analyzer.token_idx += 1;
            }
        }
    }
}

fn resolve_item(analyzer: &mut Analyzer, token: &String, range: Range, document: &Document) {
    if range.start.line != analyzer.line_nb || analyzer.token_idx == 0 {
        analyzer.line_nb = range.start.line;
        analyzer.previous_items.clear();
    }
    let full_key = format!(
        "{}-{}-{}",
        analyzer.scope.mm_es_key(),
        analyzer.scope.func_key(),
        token
    );
    let semi_key = format!("{}-{}", analyzer.scope.mm_es_key(), token);
    let mid_key = format!("{}-{}", analyzer.scope.func_key(), token);

    let item = analyzer
        .tokens_map
        .get(&full_key)
        .or_else(|| analyzer.tokens_map.get(&mid_key))
        .or_else(|| analyzer.tokens_map.get(&semi_key))
        .or_else(|| analyzer.tokens_map.get(token));

    if item.is_some() {
        let item = item.unwrap();
        let reference = Location {
            uri: document.uri.clone(),
            range,
        };
        item.lock().unwrap().push_reference(reference);
        analyzer.previous_items.push(item.clone());
        return;
    }

    if range.start.character > 0 && analyzer.previous_items.len() > 0 {
        let char = analyzer.line().as_bytes()[(range.start.character - 1) as usize] as char;
        if char != ':' && char != '.' {
            return;
        }
        let mut item: Option<Arc<Mutex<SPItem>>> = None;
        for parent in analyzer.previous_items.iter().rev() {
            let parent = parent.lock().unwrap().clone();
            match &parent {
                SPItem::EnumStruct(es) => {
                    // Enum struct scope operator (::).
                    item = analyzer.get(&format!("{}-{}", es.name, token));
                    if item.is_some() {
                        break;
                    }
                }
                SPItem::Methodmap(mm) => {
                    // Methodmap static method.
                    item = analyzer.get(&format!("{}-{}", mm.name, token));
                    if item.is_some() {
                        break;
                    }
                }
                _ => {}
            }
            item = analyzer.get(&format!("{}-{}", parent.type_(), token));
            if item.is_some() {
                break;
            }
            for inherit in find_inherit(&analyzer.all_items, &parent).into_iter() {
                item = analyzer.get(&format!("{}-{}", inherit.lock().unwrap().name(), token));
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
            range,
        };
        item.lock().unwrap().push_reference(reference);
        analyzer.previous_items.push(item.clone());
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
        if reference.uri.ne(&uri) {
            new_references.push(reference.clone());
        }
    }
    item_lock.set_new_references(new_references);
}
