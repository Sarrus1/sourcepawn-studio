use lazy_static::lazy_static;
use regex::Regex;
use tree_sitter::Node;

use crate::{
    document::{Deprecated, Walker},
    utils::ts_range_to_lsp_range,
};

pub fn parse_deprecated(node: Node, source: &str, walker: &mut Walker) {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"#pragma\s+deprecated(.*)").unwrap();
    }
    let text = node.utf8_text(&source.as_bytes()).unwrap();
    match RE.captures(text) {
        Some(caps) => match caps.get(1) {
            Some(text) => walker.deprecated.push(Deprecated {
                text: text.as_str().to_string(),
                range: ts_range_to_lsp_range(&node.range()),
            }),
            None => {}
        },
        None => {}
    };
}
