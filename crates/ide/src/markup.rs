//! Markdown formatting.
//!
//! Sometimes, we want to display a "rich text" in the UI. At the moment, we use
//! markdown for this purpose. It doesn't feel like a right option, but that's
//! what is used by LSP, so let's keep it simple.
use std::fmt;

#[derive(Default, Debug)]
pub struct Markup {
    text: String,
}

impl From<Markup> for String {
    fn from(markup: Markup) -> Self {
        markup.text
    }
}

impl From<String> for Markup {
    fn from(text: String) -> Self {
        Markup { text }
    }
}

impl fmt::Display for Markup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.text, f)
    }
}

impl Markup {
    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }

    pub fn fenced_block(contents: impl fmt::Display) -> Markup {
        format!("```sourcepawn\n{contents}\n```").into()
    }
}

impl From<Markup> for lsp_types::Documentation {
    fn from(val: Markup) -> Self {
        lsp_types::Documentation::MarkupContent(lsp_types::MarkupContent {
            kind: lsp_types::MarkupKind::Markdown,
            value: val.to_string(),
        })
    }
}
