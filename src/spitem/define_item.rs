use std::sync::Arc;

use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, Hover, HoverContents, HoverParams,
    LanguageString, MarkedString, Range, Url,
};

use crate::{providers::hover::description::Description, utils::uri_to_file_name};

use super::Location;

#[derive(Debug, Clone)]
/// SPItem representation of a SourcePawn define.
pub struct DefineItem {
    /// Name of the define.
    pub name: String,

    /// Range of the name of the define.
    pub range: Range,

    /// Value of the define.
    pub value: String,

    /// Range of the whole define, including its value.
    pub full_range: Range,

    /// Description of the define.
    pub description: Description,

    /// Uri of the file where the define is declared.
    pub uri: Arc<Url>,

    /// References to this define.
    pub references: Vec<Location>,
}

impl DefineItem {
    /// Return a [CompletionItem](lsp_types::CompletionItem) from an [DefineItem].
    ///
    /// # Arguments
    ///
    /// * `_params` - [CompletionParams](lsp_types::CompletionParams) of the request.
    pub(crate) fn to_completion(&self, _params: &CompletionParams) -> Option<CompletionItem> {
        Some(CompletionItem {
            label: self.name.to_string(),
            kind: Some(CompletionItemKind::CONSTANT),
            detail: uri_to_file_name(&self.uri),
            ..Default::default()
        })
    }

    /// Return a [Hover] from an [DefineItem].
    ///
    /// # Arguments
    ///
    /// * `_params` - [HoverParams] of the request.
    pub(crate) fn to_hover(&self, _params: &HoverParams) -> Option<Hover> {
        Some(Hover {
            contents: HoverContents::Array(vec![
                self.formatted_text(),
                MarkedString::String(self.description.to_md()),
            ]),
            range: None,
        })
    }

    /// Formatted representation of the define.
    ///
    /// # Exemple
    ///
    /// `#define FOO 1`
    fn formatted_text(&self) -> MarkedString {
        MarkedString::LanguageString(LanguageString {
            language: "sourcepawn".to_string(),
            value: format!("#define {} {}", self.name, self.value)
                .trim()
                .to_string(),
        })
    }
}
