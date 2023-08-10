use lsp_types::Range;
use tree_sitter::Node;

use crate::utils::ts_range_to_lsp_range;

#[derive(Debug)]
pub struct Comment {
    pub text: String,
    pub range: Range,
}

impl Comment {
    pub fn new(node: Node, source: &str) -> Self {
        Self {
            text: node
                .utf8_text(source.as_bytes())
                .unwrap_or_default()
                .to_string(),
            range: ts_range_to_lsp_range(&node.range()),
        }
    }
}
