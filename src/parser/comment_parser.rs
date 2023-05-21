use std::{
    str::Utf8Error,
    sync::{Arc, RwLock},
};

use lazy_static::lazy_static;
use lsp_types::Range;
use regex::Regex;
use tree_sitter::Node;

use crate::{
    document::Walker, providers::hover::description::Description, spitem::SPItem,
    utils::ts_range_to_lsp_range,
};

impl Walker {
    pub fn push_comment(&mut self, node: Node, source: &str) {
        self.comments.push(Comment::new(node, source));
    }

    pub fn push_deprecated(&mut self, node: Node, source: &str) {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"#pragma\s+deprecated(.*)").unwrap();
        }
        let text = node.utf8_text(source.as_bytes()).unwrap();
        if let Some(caps) = RE.captures(text) {
            if let Some(text) = caps.get(1) {
                self.deprecated.push(Deprecated {
                    text: text.as_str().to_string(),
                    range: ts_range_to_lsp_range(&node.range()),
                })
            }
        };
    }

    pub fn push_inline_comment(&mut self, items: &[Arc<RwLock<SPItem>>]) {
        if let Some(item) = items.last() {
            let description = self
                .find_doc(item.read().unwrap().range().end.line as usize, true)
                .unwrap();
            match &mut *item.write().unwrap() {
                SPItem::EnumMember(enum_member_item) => {
                    enum_member_item.description = description;
                }
                SPItem::Variable(variable_item) => {
                    variable_item.description = description;
                }
                SPItem::Define(define_item) => {
                    define_item.description = description;
                }
                _ => {}
            }
        }
    }

    pub fn find_doc(&mut self, end_row: usize, trailing: bool) -> Result<Description, Utf8Error> {
        let mut end_row = end_row as u32;
        let mut dep: Option<String> = None;
        let mut text: Vec<String> = vec![];
        for deprecated in self.deprecated.iter().rev() {
            if end_row == deprecated.range.end.line + 1 {
                dep = Some(deprecated.text.trim().to_string());
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

        for comment in self.comments.iter().rev() {
            if trailing {
                if end_row == comment.range.start.line {
                    let comment_text = comment.text.clone();
                    text.push(comment_to_doc(&comment_text));
                    break;
                }
            } else if end_row == comment.range.end.line + offset {
                let comment_text = comment.text.clone();
                text.push(comment_to_doc(&comment_text));
                end_row = comment.range.start.line;
            } else {
                break;
            }
        }
        if !trailing {
            self.comments.clear();
        }
        let doc = Description {
            text: text.join(""),
            deprecated: dep,
        };

        Ok(doc)
    }
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

#[derive(Debug)]
pub struct Comment {
    pub text: String,
    pub range: Range,
}

impl Comment {
    pub fn new(node: Node, source: &str) -> Self {
        Self {
            text: node.utf8_text(source.as_bytes()).unwrap().to_string(),
            range: ts_range_to_lsp_range(&node.range()),
        }
    }
}

#[derive(Debug)]
pub struct Deprecated {
    pub text: String,
    pub range: Range,
}
