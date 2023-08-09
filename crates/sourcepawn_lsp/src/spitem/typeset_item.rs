use parking_lot::RwLock;
use std::sync::Arc;

use super::{Location, SPItem};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionItemTag,
    CompletionParams, DocumentSymbol, GotoDefinitionParams, Hover, HoverContents, HoverParams,
    LanguageString, LocationLink, MarkedString, Range, SymbolKind, SymbolTag, Url,
};

use crate::{providers::hover::description::Description, utils::uri_to_file_name};

#[derive(Debug, Clone)]
/// SPItem representation of a SourcePawn typeset/funcenum, which can be converted to a
/// [CompletionItem](lsp_types::CompletionItem), [Location](lsp_types::Location), etc.
pub struct TypesetItem {
    /// Name of the typeset.
    pub name: String,

    /// Range of the name of the typeset.
    pub range: Range,

    /// User visible range of the name of the typeset.
    pub v_range: Range,

    /// Range of the whole typeset.
    pub full_range: Range,

    /// User visible range of the whole typeset.
    pub v_full_range: Range,

    /// Description of the typeset.
    pub description: Description,

    /// Uri of the file where the typeset is declared.
    pub uri: Arc<Url>,

    /// References to this typeset.
    pub references: Vec<Location>,

    /// Parameters of the typeset.
    pub children: Vec<Arc<RwLock<SPItem>>>,
}

impl TypesetItem {
    fn is_deprecated(&self) -> bool {
        self.description.deprecated.is_some()
    }

    /// Return a [CompletionItem](lsp_types::CompletionItem) from a [TypesetItem].
    ///
    /// # Arguments
    ///
    /// * `_params` - [CompletionParams](lsp_types::CompletionParams) of the request.
    pub(crate) fn to_completion(&self, params: &CompletionParams) -> Option<CompletionItem> {
        let mut tags = vec![];
        if self.is_deprecated() {
            tags.push(CompletionItemTag::DEPRECATED);
        }

        Some(CompletionItem {
            label: self.name.to_string(),
            kind: Some(CompletionItemKind::INTERFACE),
            tags: Some(tags),
            label_details: Some(CompletionItemLabelDetails {
                detail: None,
                description: if *self.uri != params.text_document_position.text_document.uri {
                    uri_to_file_name(&self.uri)
                } else {
                    None
                },
            }),
            deprecated: Some(self.is_deprecated()),
            data: Some(serde_json::Value::String(self.key())),
            ..Default::default()
        })
    }

    /// Return a [Hover] from a [TypesetItem].
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

    /// Return a [LocationLink] from a [TypesetItem].
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

    /// Return a [DocumentSymbol] from a [TypesetItem].
    pub(crate) fn to_document_symbol(&self) -> Option<DocumentSymbol> {
        let mut tags = vec![];
        if self.description.deprecated.is_some() {
            tags.push(SymbolTag::DEPRECATED);
        }
        #[allow(deprecated)]
        Some(DocumentSymbol {
            name: self.name.to_string(),
            detail: None,
            kind: SymbolKind::NAMESPACE,
            tags: Some(tags),
            range: self.v_full_range,
            deprecated: None,
            selection_range: self.v_range,
            children: Some(
                self.children
                    .iter()
                    .filter_map(|child| child.read().to_document_symbol())
                    .collect(),
            ),
        })
    }

    /// Return a vector of [CompletionItem] of all the [TypedefItem](super::typedef_item::TypedefItem)
    /// of a [TypesetItem] for a callback completion.
    ///
    /// # Arguments
    ///
    /// * `range` - [Range] of the "$" that will be replaced.
    pub(crate) fn to_snippet_completion(&self, range: Range) -> Vec<CompletionItem> {
        let mut res = vec![];
        for child in self.children.iter() {
            if let SPItem::Typedef(typedef_item) = &*child.read() {
                if let Some(completion) = typedef_item.to_snippet_completion(range) {
                    res.push(completion);
                }
            }
        }

        res
    }

    /// Return a key to be used as a unique identifier in a map containing all the items.
    pub(crate) fn key(&self) -> String {
        self.name.clone()
    }

    /// Formatted representation of a [TypesetItem].
    ///
    /// # Exemple
    ///
    /// `typeset EventHook`
    pub(crate) fn formatted_text(&self) -> String {
        format!("typeset {}", self.name)
    }
}
