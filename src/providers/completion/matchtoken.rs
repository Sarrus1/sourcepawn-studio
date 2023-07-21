use lsp_types::Range;

/// Represent a word in a line, with its text and [range](lsp_types::Range) in the document.
#[derive(Debug)]
pub(super) struct MatchToken {
    /// Text of the word.
    pub _text: String,

    /// [Range](lsp_types::Range) of the word.
    pub range: Range,
}
