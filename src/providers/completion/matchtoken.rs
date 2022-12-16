use lsp_types::Range;

/// Represent a word in a line, with its text and [range](lsp_types::Range) in the document.
#[derive(Debug)]
pub(super) struct MatchToken {
    pub _text: String,
    pub range: Range,
}
