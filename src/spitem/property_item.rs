use std::sync::{Arc, RwLock, Weak};

use super::Location;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionItemTag,
    CompletionParams, DocumentSymbol, GotoDefinitionParams, Hover, HoverContents, HoverParams,
    LanguageString, LocationLink, MarkedString, Range, SymbolKind, SymbolTag, Url,
};

use crate::providers::hover::description::Description;

use super::SPItem;

#[derive(Debug, Clone)]
/// SPItem representation of a SourcePawn property, which can be converted to a
/// [CompletionItem](lsp_types::CompletionItem), [Location](lsp_types::Location), etc.
pub struct PropertyItem {
    /// Name of the property.
    pub name: String,

    /// Parent of the property.
    pub parent: Weak<RwLock<SPItem>>,

    /// Type of the property.
    pub type_: String,

    /// Range of the name of the property.
    pub range: Range,

    /// User visible range of the name of the property.
    pub v_range: Range,

    /// Range of the whole property, including its block.
    pub full_range: Range,

    /// User visible range of the whole property, including its block.
    pub v_full_range: Range,

    /// Description of the property.
    pub description: Description,

    /// Uri of the file where the property is declared.
    pub uri: Arc<Url>,

    /// References to this property.
    pub references: Vec<Location>,
}

impl PropertyItem {
    fn is_deprecated(&self) -> bool {
        self.description.deprecated.is_some()
    }

    /// Return a [CompletionItem](lsp_types::CompletionItem) from a [PropertyItem].
    ///
    /// # Arguments
    ///
    /// * `params` - [CompletionParams](lsp_types::CompletionParams) of the request.
    /// * `request_method` - Whether we are requesting method completions or not.
    pub(crate) fn to_completion(
        &self,
        _params: &CompletionParams,
        request_method: bool,
    ) -> Option<CompletionItem> {
        // Don't return a property if non method items are requested.
        if !request_method {
            return None;
        }

        let mut tags = vec![];
        if self.is_deprecated() {
            tags.push(CompletionItemTag::DEPRECATED);
        }

        Some(CompletionItem {
            label: self.name.to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            tags: Some(tags),
            label_details: Some(CompletionItemLabelDetails {
                detail: Some(self.type_.clone()),
                description: Some(format!(
                    "{}::{}",
                    self.parent.upgrade().unwrap().read().unwrap().name(),
                    self.name
                )),
            }),
            deprecated: Some(self.is_deprecated()),
            data: Some(serde_json::Value::String(self.key())),
            ..Default::default()
        })
    }

    /// Return a [Hover] from a [PropertyItem].
    ///
    /// # Arguments
    ///
    /// * `_params` - [HoverParams] of the request.
    pub(crate) fn to_hover(&self, _params: &HoverParams) -> Option<Hover> {
        Some(Hover {
            contents: HoverContents::Array(vec![
                MarkedString::LanguageString(LanguageString {
                    language: "sourcepawn".to_string(),
                    value: self.formatted_text(),
                }),
                MarkedString::String(self.description.to_md()),
            ]),
            range: None,
        })
    }

    /// Return a [LocationLink] from a [PropertyItem].
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
            kind: SymbolKind::PROPERTY,
            tags: Some(tags),
            range: self.v_full_range,
            deprecated: None,
            selection_range: self.v_range,
            children: None,
        })
    }

    /// Return a key to be used as a unique identifier in a map containing all the items.
    pub(crate) fn key(&self) -> String {
        format!(
            "{}-{}",
            self.parent.upgrade().unwrap().read().unwrap().key(),
            self.name
        )
    }

    /// Formatted representation of a [PropertyItem].
    ///
    /// # Exemple
    ///
    /// `void OnPluginStart()`
    pub(crate) fn formatted_text(&self) -> String {
        format!(
            "{} {}",
            self.parent.upgrade().unwrap().read().unwrap().name(),
            self.name
        )
    }
}
