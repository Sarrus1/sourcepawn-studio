use lsp_types::Range;

#[derive(Debug)]
pub struct Deprecated {
    pub text: String,
    pub range: Range,
}
