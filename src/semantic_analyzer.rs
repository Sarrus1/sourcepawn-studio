use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use lazy_static::lazy_static;
use lsp_types::{Range, Url};
use tree_sitter::{Node, Query, QueryCapture, QueryCursor};

lazy_static! {
    static ref SYMBOL_QUERY: Query = {
        Query::new(
            tree_sitter_sourcepawn::language(),
            "[(symbol) @symbol (this) @symbol]",
        )
        .unwrap()
    };
}
use crate::{
    document::Document,
    spitem::{get_all_items, Location, SPItem},
    store::Store,
    utils::{range_contains_range, ts_range_to_lsp_range},
};

#[derive(Debug, Default)]
pub struct Scope {
    pub func: Option<Arc<Mutex<SPItem>>>,
    pub mm_es: Option<Arc<Mutex<SPItem>>>,
}

impl Scope {
    fn func_full_range(&self) -> Range {
        self.func
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .full_range()
            .unwrap()
    }

    fn mm_es_full_range(&self) -> Range {
        self.mm_es
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .full_range()
            .unwrap()
    }

    pub fn update_func(
        &mut self,
        range: Range,
        func_idx: &mut usize,
        funcs_in_file: &Vec<Arc<Mutex<SPItem>>>,
    ) {
        // Do not update the function, we are still in its scope.
        if self.func.is_some() && range_contains_range(&self.func_full_range(), &range) {
            return;
        }

        if *func_idx >= funcs_in_file.len() {
            self.func = None;
            return;
        }

        let next_func_range = funcs_in_file[*func_idx]
            .lock()
            .unwrap()
            .full_range()
            .unwrap();

        if range_contains_range(&next_func_range, &range) {
            self.func = Some(funcs_in_file[*func_idx].clone());
            *func_idx += 1;
        } else {
            self.func = None;
        }
    }

    pub fn update_mm_es(
        &mut self,
        range: Range,
        mm_es_idx: &mut usize,
        mm_es_in_file: &Vec<Arc<Mutex<SPItem>>>,
    ) {
        // Do not update the function, we are still in its scope.
        if self.mm_es.is_some() && range_contains_range(&self.mm_es_full_range(), &range) {
            return;
        }

        if *mm_es_idx >= mm_es_in_file.len() {
            self.mm_es = None;
            return;
        }

        let next_mm_es_range = mm_es_in_file[*mm_es_idx]
            .lock()
            .unwrap()
            .full_range()
            .unwrap();

        if range_contains_range(&next_mm_es_range, &range) {
            self.mm_es = Some(mm_es_in_file[*mm_es_idx].clone());
            *mm_es_idx += 1;
        } else {
            self.mm_es = None;
        }
    }

    pub fn func_key(&self) -> String {
        if self.func.is_none() {
            return "".to_string();
        }
        self.func.clone().unwrap().lock().unwrap().name()
    }

    pub fn mm_es_key(&self) -> String {
        if self.mm_es.is_none() {
            return "".to_string();
        }
        self.mm_es.clone().unwrap().lock().unwrap().name()
    }
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

#[derive(Debug, Default)]
pub struct Analyzer {
    pub lines: Vec<String>,
    pub all_items: Vec<Arc<Mutex<SPItem>>>,
    pub previous_items: Vec<Arc<Mutex<SPItem>>>,
    pub line_nb: u32,
    pub tokens_map: HashMap<String, Arc<Mutex<SPItem>>>,
    pub funcs_in_file: Vec<Arc<Mutex<SPItem>>>,
    pub mm_es_in_file: Vec<Arc<Mutex<SPItem>>>,
    pub scope: Scope,
    pub func_idx: usize,
    pub mm_es_idx: usize,
    pub token_idx: u32,
}

impl Analyzer {
    pub fn new(all_items: Vec<Arc<Mutex<SPItem>>>, document: &Document) -> Self {
        let mut tokens_map: HashMap<String, Arc<Mutex<SPItem>>> = HashMap::new();
        let mut funcs_in_file = vec![];
        let mut mm_es_in_file = vec![];

        for item in all_items.iter() {
            purge_references(item, &document.uri);
            match &*item.lock().unwrap() {
                // Match variables
                SPItem::Variable(variable_item) => match &variable_item.parent {
                    // Match non global variables
                    Some(variable_item_parent) => match &*variable_item_parent.lock().unwrap() {
                        // Match variables in a function or method
                        SPItem::Function(variable_item_parent_function) => {
                            let key = format!(
                                "{}-{}",
                                variable_item_parent_function.name, variable_item.name
                            );
                            tokens_map.insert(key, item.clone());
                        }
                        // Match variables as enum struct fields
                        SPItem::EnumStruct(variable_item_parent_enum_struct) => {
                            let key = format!(
                                "{}-{}",
                                variable_item_parent_enum_struct.name, variable_item.name
                            );
                            tokens_map.insert(key, item.clone());
                        }
                        _ => {}
                    },
                    None => {
                        tokens_map.insert(variable_item.name.to_string(), item.clone());
                    }
                },
                SPItem::Function(function_item) => match &function_item.parent {
                    Some(method_item_parent) => match &*method_item_parent.lock().unwrap() {
                        SPItem::Methodmap(method_item_parent) => {
                            let key = format!("{}-{}", method_item_parent.name, function_item.name);
                            tokens_map.insert(key, item.clone());
                        }
                        SPItem::EnumStruct(method_item_parent) => {
                            let key = format!("{}-{}", method_item_parent.name, function_item.name);
                            tokens_map.insert(key, item.clone());
                        }
                        _ => {}
                    },
                    None => {
                        if function_item.uri.eq(&document.uri) {
                            funcs_in_file.push(item.clone());
                        }
                        tokens_map.insert(function_item.name.to_string(), item.clone());
                    }
                },
                SPItem::Methodmap(methodmap_item) => {
                    if methodmap_item.uri.eq(&document.uri) {
                        mm_es_in_file.push(item.clone());
                    }
                    tokens_map.insert(methodmap_item.name.to_string(), item.clone());
                }
                SPItem::EnumStruct(enum_struct_item) => {
                    if enum_struct_item.uri.eq(&document.uri) {
                        mm_es_in_file.push(item.clone());
                    }
                    tokens_map.insert(enum_struct_item.name.to_string(), item.clone());
                }
                SPItem::Enum(enum_item) => {
                    tokens_map.insert(enum_item.name.to_string(), item.clone());
                }
                SPItem::Define(define_item) => {
                    tokens_map.insert(define_item.name.to_string(), item.clone());
                }
                SPItem::EnumMember(enum_member_item) => {
                    tokens_map.insert(enum_member_item.name.to_string(), item.clone());
                } // TODO: add typedef and typeset here
                _ => {}
            }
        }

        Self {
            tokens_map,
            all_items,
            funcs_in_file,
            mm_es_in_file,
            lines: document
                .text
                .lines()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            ..Default::default()
        }
    }

    pub fn update_scope(&mut self, range: Range) {
        self.scope
            .update_func(range, &mut self.func_idx, &self.funcs_in_file);
        self.scope
            .update_mm_es(range, &mut self.mm_es_idx, &self.mm_es_in_file);
    }

    pub fn line(&self) -> &String {
        &self.lines[self.line_nb as usize]
    }

    pub fn get(&self, key: &String) -> Option<Arc<Mutex<SPItem>>> {
        match self.tokens_map.get(key) {
            Some(res) => Some(res.clone()),
            None => None,
        }
    }
}

pub fn find_references(store: &Store, root_node: Node, document: Document) {
    let all_items = get_all_items(store);
    if all_items.is_none() {
        return;
    }
    let all_items = all_items.unwrap();
    let mut analyzer = Analyzer::new(all_items, &document);
    let mut cursor = QueryCursor::new();
    let matches = cursor.captures(&SYMBOL_QUERY, root_node, document.text.as_bytes());
    for (match_, _) in matches {
        for capture in match_.captures.iter() {
            let (token, range) = capture_text_range(capture, &document.text);

            analyzer.update_scope(range);
            resolve_item(&mut analyzer, &token, range, &document);

            analyzer.token_idx += 1;
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

struct Inherit {
    item: Option<Arc<Mutex<SPItem>>>,
}

impl Iterator for Inherit {
    type Item = Arc<Mutex<SPItem>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.item.is_none() {
            return None;
        }
        let item = self.item.clone().unwrap();
        let item = item.lock().unwrap();
        match &*item {
            SPItem::Methodmap(inherit) => match &inherit.parent {
                Some(parent) => {
                    self.item = Some(parent.clone());
                    return Some(parent.clone());
                }
                None => return None,
            },
            _ => return None,
        }
    }
}

fn find_inherit(all_items: &Vec<Arc<Mutex<SPItem>>>, parent: &SPItem) -> Inherit {
    let mut inherit = None;
    for item_ in all_items.iter() {
        let item_lock = item_.lock().unwrap();
        match &*item_lock {
            SPItem::Methodmap(mm_item) => {
                if mm_item.name == parent.type_() {
                    inherit = Some(item_.clone());
                    break;
                }
            }
            _ => {}
        }
    }

    Inherit { item: inherit }
}

fn purge_references(item: &Arc<Mutex<SPItem>>, uri: &Arc<Url>) {
    let mut new_references = vec![];
    let item_lock = item.lock().unwrap();
    let old_references = item_lock.references();
    if old_references.is_none() {
        return;
    }
    let old_references = old_references.unwrap();
    for reference in old_references {
        if reference.uri.eq(&uri) {
            new_references.push(reference);
        }
    }
}
