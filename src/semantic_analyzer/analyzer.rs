use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use lsp_types::Range;

use crate::{document::Document, spitem::SPItem};

use super::{purge_references, scope::Scope};

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
                }
                SPItem::Property(property_item) => match &*property_item.parent.lock().unwrap() {
                    SPItem::Methodmap(property_item_parent) => {
                        let key = format!("{}-{}", property_item_parent.name, property_item.name);
                        tokens_map.insert(key, item.clone());
                    }
                    _ => { /* Won't happen */ }
                },
                // TODO: add typedef and typeset here
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
