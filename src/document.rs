use std::{
    collections::HashMap,
    collections::HashSet,
    str::Utf8Error,
    sync::{Arc, RwLock},
};

use derive_new::new;
use lazy_static::lazy_static;
use lsp_types::Range;
use lsp_types::Url;
use regex::Regex;
use tree_sitter::{Node, Query, QueryCursor};

use crate::{
    parser::comment_parser::{Comment, Deprecated},
    providers::hover::description::Description,
    spitem::SPItem,
    utils::ts_range_to_lsp_range,
};

lazy_static! {
    static ref SYMBOL_QUERY: Query = {
        Query::new(
            tree_sitter_sourcepawn::language(),
            "[(symbol) @symbol (this) @symbol]",
        )
        .unwrap()
    };
}

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub range: Range,
}

impl Token {
    pub fn new(node: Node, source: &String) -> Self {
        Self {
            text: node.utf8_text(source.as_bytes()).unwrap().to_string(),
            range: ts_range_to_lsp_range(&node.range()),
        }
    }
}

#[derive(Debug, Clone, new)]
pub struct Document {
    pub uri: Arc<Url>,
    pub text: String,
    #[new(default)]
    pub sp_items: Vec<Arc<RwLock<SPItem>>>,
    #[new(default)]
    pub includes: HashMap<Url, Token>,
    #[new(value = "false")]
    pub parsed: bool,
    #[new(value = "vec![]")]
    pub tokens: Vec<Arc<Token>>,
    #[new(default)]
    pub missing_includes: HashMap<String, Range>,
    #[new(default)]
    pub unresolved_tokens: HashSet<String>,
    #[new(default)]
    pub declarations: HashMap<String, Arc<RwLock<SPItem>>>,
}

pub struct Walker {
    pub comments: Vec<Comment>,
    pub deprecated: Vec<Deprecated>,
    pub anon_enum_counter: u32,
}

impl Document {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn extract_tokens(&mut self, root_node: Node) {
        let mut cursor = QueryCursor::new();
        let matches = cursor.captures(&SYMBOL_QUERY, root_node, self.text.as_bytes());
        for (match_, _) in matches {
            for capture in match_.captures.iter() {
                self.tokens
                    .push(Arc::new(Token::new(capture.node, &self.text)));
            }
        }
    }

    pub fn line(&self, line_nb: u32) -> Option<&str> {
        for (i, line) in self.text.lines().enumerate() {
            if i == line_nb as usize {
                return Some(line);
            }
        }

        None
    }

    pub fn sp_items(&self) -> Vec<Arc<RwLock<SPItem>>> {
        let mut sp_items = vec![];
        for item in self.sp_items.iter() {
            sp_items.push(item.clone());
        }

        sp_items
    }

    pub fn sp_items_flat(&self) -> Vec<Arc<RwLock<SPItem>>> {
        let mut sp_items = vec![];
        for item in self.sp_items.iter() {
            sp_items.push(item.clone());
            match &*item.read().unwrap() {
                SPItem::Function(function_item) => {
                    for child_item in function_item.children.iter() {
                        sp_items.push(child_item.clone())
                    }
                }
                SPItem::Enum(enum_item) => {
                    for child_item in enum_item.children.iter() {
                        sp_items.push(child_item.clone())
                    }
                }
                SPItem::EnumStruct(es_item) => {
                    for child_item in es_item.children.iter() {
                        sp_items.push(child_item.clone());
                        match &*child_item.read().unwrap() {
                            SPItem::Function(method_item) => {
                                for sub_child_item in method_item.children.iter() {
                                    sp_items.push(sub_child_item.clone());
                                }
                            }
                            SPItem::EnumMember(_)
                            | SPItem::Typedef(_)
                            | SPItem::Typeset(_)
                            | SPItem::Variable(_)
                            | SPItem::Property(_)
                            | SPItem::Include(_)
                            | SPItem::Methodmap(_)
                            | SPItem::Enum(_)
                            | SPItem::EnumStruct(_)
                            | SPItem::Define(_) => {}
                        }
                    }
                }
                SPItem::Methodmap(mm_item) => {
                    for child_item in mm_item.children.iter() {
                        sp_items.push(child_item.clone());
                        match &*child_item.read().unwrap() {
                            SPItem::Function(method_item) => {
                                for sub_child_item in method_item.children.iter() {
                                    sp_items.push(sub_child_item.clone());
                                }
                            }
                            SPItem::EnumMember(_)
                            | SPItem::Typedef(_)
                            | SPItem::Typeset(_)
                            | SPItem::Property(_)
                            | SPItem::Variable(_)
                            | SPItem::Include(_)
                            | SPItem::Methodmap(_)
                            | SPItem::Enum(_)
                            | SPItem::EnumStruct(_)
                            | SPItem::Define(_) => {}
                        }
                    }
                }
                SPItem::Typeset(ts_item) => {
                    for child_item in ts_item.children.iter() {
                        sp_items.push(child_item.clone())
                    }
                }
                SPItem::Variable(_)
                | SPItem::Typedef(_)
                | SPItem::EnumMember(_)
                | SPItem::Property(_)
                | SPItem::Include(_)
                | SPItem::Define(_) => {}
            }
        }

        sp_items
    }
}

pub fn find_doc(walker: &mut Walker, end_row: usize) -> Result<Description, Utf8Error> {
    let mut end_row = end_row as u32;
    let mut dep: Option<String> = None;
    let mut text: Vec<String> = vec![];

    for deprecated in walker.deprecated.iter().rev() {
        if end_row == deprecated.range.end.line + 1 {
            dep = Some(deprecated.text.clone());
            break;
        }
        if end_row > deprecated.range.end.line {
            break;
        }
    }
    let mut offset = 1;
    if dep.is_some() {
        offset = 2;
    }

    for comment in walker.comments.iter().rev() {
        if end_row == comment.range.end.line + offset {
            let comment_text = comment.text.clone();
            text.push(comment_to_doc(&comment_text));
            end_row = comment.range.start.line;
        } else {
            break;
        }
    }
    walker.comments.clear();
    let doc = Description {
        text: text.join(""),
        deprecated: dep,
    };

    Ok(doc)
}

fn comment_to_doc(text: &str) -> String {
    lazy_static! {
        static ref RE1: Regex = Regex::new(r"^/\*\s*").unwrap();
        static ref RE2: Regex = Regex::new(r"\*/$").unwrap();
        static ref RE3: Regex = Regex::new(r"//\s*").unwrap();
    }
    let text = RE1.replace_all(text, "").into_owned();
    let text = RE2.replace_all(&text, "").into_owned();
    let text = RE3.replace_all(&text, "").into_owned();

    text
}
