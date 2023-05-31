use std::sync::{Arc, RwLock};

use fxhash::FxHashMap;
use lsp_types::Range;

use crate::{
    document::{Document, Token},
    spitem::SPItem,
};

use super::{purge_references, scope::Scope};

#[derive(Debug, Default)]
pub struct Analyzer {
    pub lines: Vec<String>,
    pub all_items: Vec<Arc<RwLock<SPItem>>>,
    pub previous_items: Vec<Arc<RwLock<SPItem>>>,
    pub line_nb: u32,
    pub tokens_map: FxHashMap<String, Arc<RwLock<SPItem>>>,
    pub funcs_in_file: Vec<Arc<RwLock<SPItem>>>,
    pub mm_es_in_file: Vec<Arc<RwLock<SPItem>>>,
    pub scope: Scope,
    pub func_idx: usize,
    pub mm_es_idx: usize,
    pub token_idx: u32,
}

impl Analyzer {
    pub fn new(all_items: Vec<Arc<RwLock<SPItem>>>, document: &Document) -> Self {
        let mut tokens_map = FxHashMap::default();
        let mut funcs_in_file = vec![];
        let mut mm_es_in_file = vec![];

        for item in all_items.iter() {
            purge_references(item, &document.uri);
            match &*item.read().unwrap() {
                // Match variables
                SPItem::Variable(variable_item) => {
                    // Global variable
                    tokens_map.insert(variable_item.key(), item.clone());
                }
                SPItem::Function(function_item) => {
                    // First level function.
                    if function_item.uri.eq(&document.uri) {
                        funcs_in_file.push(item.clone());
                    }
                    tokens_map.insert(function_item.key(), item.clone());
                    // All variables of the function.
                    for child in &function_item.children {
                        purge_references(child, &document.uri);
                        tokens_map.insert(child.read().unwrap().key(), child.clone());
                    }
                }
                SPItem::Methodmap(methodmap_item) => {
                    if methodmap_item.uri.eq(&document.uri) {
                        mm_es_in_file.push(item.clone());
                    }
                    tokens_map.insert(methodmap_item.key(), item.clone());
                    // All properties and methods of the enum struct.
                    for child in &methodmap_item.children {
                        purge_references(child, &document.uri);
                        tokens_map.insert(child.read().unwrap().key(), child.clone());
                        if let SPItem::Function(method_item) = &*child.read().unwrap() {
                            if method_item.uri.eq(&document.uri) {
                                funcs_in_file.push(child.clone());
                            }
                            // All variables of the method.
                            for sub_child in &method_item.children {
                                purge_references(sub_child, &document.uri);
                                tokens_map
                                    .insert(sub_child.read().unwrap().key(), sub_child.clone());
                            }
                        }
                    }
                }
                SPItem::EnumStruct(enum_struct_item) => {
                    if enum_struct_item.uri.eq(&document.uri) {
                        mm_es_in_file.push(item.clone());
                    }
                    tokens_map.insert(enum_struct_item.key(), item.clone());
                    // All fields and methods of the enum struct.
                    for child in &enum_struct_item.children {
                        purge_references(child, &document.uri);
                        tokens_map.insert(child.read().unwrap().key(), child.clone());
                        if let SPItem::Function(method_item) = &*child.read().unwrap() {
                            if method_item.uri.eq(&document.uri) {
                                funcs_in_file.push(child.clone());
                            }
                            // All variables of the method.
                            for sub_child in &method_item.children {
                                purge_references(sub_child, &document.uri);
                                tokens_map
                                    .insert(sub_child.read().unwrap().key(), sub_child.clone());
                            }
                        }
                    }
                }
                SPItem::Enum(enum_item) => {
                    tokens_map.insert(enum_item.key(), item.clone());
                    // All enum members of the enum.
                    for child in &enum_item.children {
                        purge_references(child, &document.uri);
                        tokens_map.insert(child.read().unwrap().key(), child.clone());
                    }
                }
                SPItem::Define(define_item) => {
                    tokens_map.insert(define_item.key(), item.clone());
                }
                SPItem::Property(property_item) => {
                    if let SPItem::Methodmap(_) =
                        &*property_item.parent.upgrade().unwrap().read().unwrap()
                    {
                        tokens_map.insert(property_item.key(), item.clone());
                    }
                }
                SPItem::Typedef(typedef_item) => {
                    tokens_map.insert(typedef_item.key(), item.clone());
                }
                SPItem::Typeset(typeset_item) => {
                    tokens_map.insert(typeset_item.key(), item.clone());
                    // All typedef members of the typeset.
                    for child in &typeset_item.children {
                        purge_references(child, &document.uri);
                        tokens_map.insert(child.read().unwrap().key(), child.clone());
                    }
                }
                SPItem::Include(_) => {}
                SPItem::EnumMember(_) => {}
            }
        }

        Self {
            tokens_map,
            all_items,
            funcs_in_file,
            mm_es_in_file,
            lines: document
                .preprocessed_text
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

    pub fn get(&self, key: &String) -> Option<Arc<RwLock<SPItem>>> {
        self.tokens_map.get(key).cloned()
    }

    pub fn update_line_context(&mut self, token: &Arc<Token>) {
        if (token.range.start.line != self.line_nb || self.token_idx == 0)
            && !token.range.start.line >= self.lines.len() as u32
        {
            self.line_nb = token.range.start.line;
            self.previous_items.clear();
        }
    }
}
