use std::sync::Arc;

use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, Hover, HoverContents, HoverParams,
    LanguageString, Location, MarkedString, Range, Url,
};

use crate::{providers::hover::description::Description, utils::uri_to_file_name};

#[derive(Debug, Clone)]
/// SPItem representation of a SourcePawn enum.
pub struct EnumItem {
    /// Name of the enum.
    pub name: String,

    /// Range of the name of the enum.
    pub range: Range,

    /// Range of the whole enum, including its block.
    pub full_range: Range,

    /// Description of the enum.
    pub description: Description,

    /// Uri of the file where the enum is declared.
    pub uri: Arc<Url>,

    /// References to this enum.
    pub references: Vec<Location>,
}

impl EnumItem {
    /// Return a [CompletionItem](lsp_types::CompletionItem) from an [EnumItem].
    ///
    /// # Arguments
    ///
    /// * `_params` - [CompletionParams](lsp_types::CompletionParams) of the request.
    pub(crate) fn to_completion(&self, _params: &CompletionParams) -> Option<CompletionItem> {
        if self.name.contains("#") {
            return None;
        }

        Some(CompletionItem {
            label: self.name.to_string(),
            kind: Some(CompletionItemKind::ENUM),
            detail: uri_to_file_name(&self.uri),
            ..Default::default()
        })
    }

    /// Return a [Hover] from an [EnumItem].
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

    /// Formatted representation of the enum.
    ///
    /// # Exemple
    ///
    /// `enum Action`
    fn formatted_text(&self) -> MarkedString {
        MarkedString::LanguageString(LanguageString {
            language: "sourcepawn".to_string(),
            value: format!("enum {}", self.name),
        })
    }
}
