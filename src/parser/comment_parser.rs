use lazy_static::lazy_static;
use lsp_types::Range;
use regex::Regex;
use tree_sitter::Node;

use crate::{document::Walker, utils::ts_range_to_lsp_range};

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
