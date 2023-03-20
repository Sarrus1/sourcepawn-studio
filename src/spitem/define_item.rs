use std::sync::Arc;

use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemTag, CompletionParams, DocumentSymbol,
    GotoDefinitionParams, Hover, HoverContents, HoverParams, LanguageString, LocationLink,
    MarkedString, Range, SymbolKind, SymbolTag, Url,
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
        let mut tags = vec![];
        if self.description.deprecated.is_some() {
            tags.push(CompletionItemTag::DEPRECATED);
        }

        Some(CompletionItem {
            label: self.name.to_string(),
            kind: Some(CompletionItemKind::CONSTANT),
            tags: Some(tags),
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

    /// Return a [LocationLink] from a [DefineItem].
    ///
    /// # Arguments
    ///
    /// * `_params` - [GotoDefinitionParams] of the request.
    pub(crate) fn to_definition(&self, _params: &GotoDefinitionParams) -> Option<LocationLink> {
        Some(LocationLink {
            target_range: self.range,
            target_uri: self.uri.as_ref().clone(),
            target_selection_range: self.range,
            origin_selection_range: None,
        })
    }

    /// Return a [DocumentSymbol] from a [DefineItem].
    pub(crate) fn to_document_symbol(&self) -> Option<DocumentSymbol> {
        let mut tags = vec![];
        if self.description.deprecated.is_some() {
            tags.push(SymbolTag::DEPRECATED);
        }
        #[allow(deprecated)]
        Some(DocumentSymbol {
            name: self.name.to_string(),
            detail: Some(self.value.to_string()),
            kind: SymbolKind::CONSTANT,
            tags: Some(tags),
            range: self.full_range,
            deprecated: None,
            selection_range: self.range,
            children: None,
        })
    }

    /// Return a key to be used as a unique identifier in a map containing all the items.
    pub(crate) fn key(&self) -> String {
        self.name.clone()
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
