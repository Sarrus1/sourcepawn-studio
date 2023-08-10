use std::sync::Arc;

use lsp_types::Range;
use syntax::utils::ts_range_to_lsp_range;
use tree_sitter::Node;

#[derive(Debug, Clone)]
pub enum SPToken {
    Symbol(Arc<Token>),
    Method((Arc<Token>, Arc<Token>)),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub range: Range,
}

impl Token {
    pub fn new(node: Node, source: &String) -> Self {
        Self {
            text: node
                .utf8_text(source.as_bytes())
                .unwrap_or_default()
                .to_string(),
            range: ts_range_to_lsp_range(&node.range()),
        }
    }
}
