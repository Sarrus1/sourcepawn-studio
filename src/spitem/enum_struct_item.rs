use std::sync::{Arc, RwLock};

use super::{Location, SPItem};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, DocumentSymbol, GotoDefinitionParams,
    Hover, HoverContents, HoverParams, LanguageString, LocationLink, MarkedString, Range,
    SymbolKind, SymbolTag, Url,
};

use crate::{providers::hover::description::Description, utils::uri_to_file_name};

#[derive(Debug, Clone)]
/// SPItem representation of a SourcePawn enum struct.
pub struct EnumStructItem {
    /// Name of the enum struct.
    pub name: String,

    /// Range of the name of the enum struct.
    pub range: Range,

    /// Range of the whole enum struct, including its block.
    pub full_range: Range,

    /// Description of the enum struct.
    pub description: Description,

    /// Uri of the file where the enum struct is declared.
    pub uri: Arc<Url>,

    /// References to this enum struct.
    pub references: Vec<Location>,

    /// Children ([FunctionItem](super::function_item::FunctionItem),
    /// [VariableItem](super::variable_item::VariableItem)) of this enum struct.
    pub children: Vec<Arc<RwLock<SPItem>>>,
}

impl EnumStructItem {
    /// Return a vector of [CompletionItem](lsp_types::CompletionItem) from an [EnumStructItem] and its children.
    ///
    /// # Arguments
    ///
    /// * `params` - [CompletionParams](lsp_types::CompletionParams) of the request.
    /// * `request_method` - Whether we are requesting method completions or not.
    pub(crate) fn to_completions(
        &self,
        params: &CompletionParams,
        request_method: bool,
    ) -> Vec<CompletionItem> {
        let mut res = vec![];
        res.push(CompletionItem {
            label: self.name.to_string(),
            kind: Some(CompletionItemKind::STRUCT),
            detail: uri_to_file_name(&self.uri),
            ..Default::default()
        });

        for child in &self.children {
            res.extend(child.read().unwrap().to_completions(params, request_method))
        }

        res
    }

    /// Return a [Hover] from an [EnumStructItem].
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

    /// Return a [LocationLink] from an [EnumStructItem].
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

    /// Return a [DocumentSymbol] from an [EnumStructItem].
    pub(crate) fn to_document_symbol(&self) -> Option<DocumentSymbol> {
        let mut tags = vec![];
        if self.description.deprecated.is_some() {
            tags.push(SymbolTag::DEPRECATED);
        }
        #[allow(deprecated)]
        Some(DocumentSymbol {
            name: self.name.to_string(),
            detail: None,
            kind: SymbolKind::STRUCT,
            tags: Some(tags),
            range: self.full_range,
            deprecated: None,
            selection_range: self.range,
            children: Some(
                self.children
                    .iter()
                    .filter_map(|child| child.read().unwrap().to_document_symbol())
                    .collect(),
            ),
        })
    }

    /// Return a key to be used as a unique identifier in a map containing all the items.
    pub(crate) fn key(&self) -> String {
        self.name.clone()
    }

    /// Formatted representation of an [EnumStructItem].
    ///
    /// # Exemple
    ///
    /// `enum struct Action`
    fn formatted_text(&self) -> MarkedString {
        MarkedString::LanguageString(LanguageString {
            language: "sourcepawn".to_string(),
            value: format!("enum struct {}", self.name),
        })
    }
}
