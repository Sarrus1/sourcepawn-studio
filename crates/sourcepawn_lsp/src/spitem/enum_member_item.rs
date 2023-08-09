use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionItemTag,
    CompletionParams, DocumentSymbol, Hover, HoverContents, HoverParams, LanguageString,
    MarkedString, Range, SymbolKind, SymbolTag, Url,
};
use lsp_types::{GotoDefinitionParams, LocationLink};
use parking_lot::RwLock;
use std::sync::{Arc, Weak};

use super::Location;
use super::SPItem;
use crate::providers::hover::description::Description;

#[derive(Debug, Clone)]
/// SPItem representation of a SourcePawn enum member.
pub struct EnumMemberItem {
    /// Name of the enum member.
    pub name: String,

    /// Range of the name of the enum member.
    pub range: Range,

    /// User visible range of the name of the enum member.
    pub v_range: Range,

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
            label_details: Some(CompletionItemLabelDetails {
                detail: None,
                description: {
                    let name = self.parent.upgrade().unwrap().read().name();
                    if name.starts_with("Enum#") {
                        None
                    } else {
                        Some(format!(
                            "{}::{}",
                            self.parent.upgrade().unwrap().read().name(),
                            self.name
                        ))
                    }
                },
            }),
            data: Some(serde_json::Value::String(self.key())),
            ..Default::default()
        })
    }

    /// Return a [Hover] from an [EnumItem].
    ///
    /// # Arguments
    ///
    /// * `_params` - [HoverParams] of the request.
    pub(crate) fn to_hover(&self, _params: &HoverParams) -> Option<Hover> {
        let mut contents = vec![MarkedString::LanguageString(LanguageString {
            language: "sourcepawn".to_string(),
            value: self.formatted_text(),
        })];
        if let Some(md_text) = self.description.to_md() {
            contents.push(MarkedString::String(md_text))
        }
        Some(Hover {
            contents: HoverContents::Array(contents),
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
            target_range: self.v_range,
            target_uri: self.uri.as_ref().clone(),
            target_selection_range: self.v_range,
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
            range: self.v_range,
            deprecated: None,
            selection_range: self.v_range,
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
    pub(crate) fn formatted_text(&self) -> String {
        let mut value = "".to_string();
        if let SPItem::Enum(parent) = &*self.parent.upgrade().unwrap().read() {
            if parent.name.contains('#') {
                value = self.name.clone()
            } else {
                value = format!("{}::{}", parent.name, self.name);
            }
        }
        value
    }
}
