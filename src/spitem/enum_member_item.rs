use std::sync::{Arc, RwLock, Weak};

use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemTag, CompletionParams, DocumentSymbol, Hover,
    HoverContents, HoverParams, LanguageString, MarkedString, Range, SymbolKind, SymbolTag, Url,
};
use lsp_types::{GotoDefinitionParams, LocationLink};

use crate::providers::hover::description::Description;

use super::Location;
use super::SPItem;

#[derive(Debug, Clone)]
/// SPItem representation of a SourcePawn enum member.
pub struct EnumMemberItem {
    /// Name of the enum member.
    pub name: String,

    /// Range of the name of the enum member.
    pub range: Range,

    /// Parent of the method. None if it's a first class function.
    pub parent: Weak<RwLock<SPItem>>,

    /// Description of the enum member.
    pub description: Description,

    /// Uri of the file where the enum member is declared.
    pub uri: Arc<Url>,

    /// References to this enum.
    pub references: Vec<Location>,
}

impl EnumMemberItem {
    /// Return a [CompletionItem](lsp_types::CompletionItem) from an [EnumMemberItem].
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
            label: self.name.clone(),
            kind: Some(CompletionItemKind::ENUM_MEMBER),
            tags: Some(tags),
            detail: Some(self.parent.upgrade().unwrap().read().unwrap().name()),
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

    /// Return a [LocationLink] from an [EnumMemberItem].
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
            detail: None,
            kind: SymbolKind::ENUM_MEMBER,
            tags: Some(tags),
            range: self.range,
            deprecated: None,
            selection_range: self.range,
            children: None,
        })
    }

    /// Return a key to be used as a unique identifier in a map containing all the items.
    pub(crate) fn key(&self) -> String {
        self.name.clone()
    }

    /// Formatted representation of the enum member.
    ///
    /// # Exemple
    ///
    /// `Plugin_Continue`
    fn formatted_text(&self) -> MarkedString {
        let mut value = "".to_string();
        if let SPItem::Enum(parent) = &*self.parent.upgrade().unwrap().read().unwrap() {
            if parent.name.contains('#') {
                value = self.name.clone()
            } else {
                value = format!("{}::{}", parent.name, self.name);
            }
        }
        MarkedString::LanguageString(LanguageString {
            language: "sourcepawn".to_string(),
            value,
        })
    }
}
