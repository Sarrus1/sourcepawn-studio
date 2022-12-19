use std::sync::{Arc, Mutex};

use super::Location;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemTag, CompletionParams, GotoDefinitionParams,
    Hover, HoverContents, HoverParams, LanguageString, LocationLink, MarkedString, Range, Url,
};

use crate::providers::hover::description::Description;

use super::SPItem;

#[derive(Debug, Clone)]
/// SPItem representation of a first order SourcePawn function, which can be converted to a
/// [CompletionItem](lsp_types::CompletionItem), [Location](lsp_types::Location), etc.
pub struct FunctionItem {
    /// Name of the function.
    pub name: String,

    /// Return type of the function.
    pub type_: String,

    /// Range of the name of the function.
    pub range: Range,

    /// Range of the whole function, including its block.
    pub full_range: Range,

    /// Description of the function.
    pub description: Description,

    /// Uri of the file where the function is declared.
    pub uri: Arc<Url>,

    /// Full function signature.
    pub detail: String,

    /// Visibility of the function.
    pub visibility: Vec<FunctionVisibility>,

    /// Definition type of the function.
    pub definition_type: FunctionDefinitionType,

    /// References to this function.
    pub references: Vec<Location>,

    /// Parameters of the function.
    pub params: Vec<Arc<Mutex<SPItem>>>,

    /// Parent of the method. None if it's a first class function.
    pub parent: Option<Arc<Mutex<SPItem>>>,
}

impl FunctionItem {
    fn is_deprecated(&self) -> bool {
        self.description.deprecated.is_some()
    }

    /// Return a [CompletionItem](lsp_types::CompletionItem) from a [FunctionItem].
    ///
    /// If the conditions are not appropriate (ex: asking for a static outside of its file), return None.
    ///
    /// # Arguments
    ///
    /// * `params` - [CompletionParams](lsp_types::CompletionParams) of the request.
    pub(crate) fn to_completion(
        &self,
        params: &CompletionParams,
        request_method: bool,
    ) -> Option<CompletionItem> {
        // Don't return a method if non method items are requested.
        if !request_method && self.parent.is_some() {
            return None;
        }

        let mut tags = vec![];
        if self.is_deprecated() {
            tags.push(CompletionItemTag::DEPRECATED);
        }

        // Don't return a CompletionItem if it's a static and the request did not come from the file
        // of the declaration.
        if self.visibility.contains(&FunctionVisibility::Static)
            && params.text_document_position.text_document.uri.to_string() != self.uri.to_string()
        {
            return None;
        }

        Some(CompletionItem {
            label: self.name.to_string(),
            kind: Some(self.completion_kind()),
            tags: Some(tags),
            detail: Some(self.type_.to_string()),
            deprecated: Some(self.is_deprecated()),
            ..Default::default()
        })
    }

    /// Return a [Hover] from a [FunctionItem].
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

    /// Return a [LocationLink] from a [FunctionItem].
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

    /// Formatted representation of a [FunctionItem].
    ///
    /// # Exemple
    ///
    /// `void OnPluginStart()`
    fn formatted_text(&self) -> MarkedString {
        MarkedString::LanguageString(LanguageString {
            language: "sourcepawn".to_string(),
            value: format!("{} {}()", self.type_, self.name),
        })
    }

    /// Returns a [CompletionItemKind](lsp_types::CompletionItemKind) depending on
    /// if it is a function or a method.
    fn completion_kind(&self) -> CompletionItemKind {
        if self.parent.is_some() {
            CompletionItemKind::METHOD
        } else {
            CompletionItemKind::FUNCTION
        }
    }
}

/// Visibility of a SourcePawn function.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FunctionVisibility {
    Public,
    Static,
    Stock,
}

/// Definition type of a SourcePawn function.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FunctionDefinitionType {
    Forward,
    Native,
    None,
}
