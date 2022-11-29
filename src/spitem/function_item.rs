use std::sync::Arc;

use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemTag, CompletionParams, Location, Range, Url,
};

use crate::providers::hover::description::Description;

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
    // params: VariableItem[];
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
    pub(crate) fn to_completion(&self, params: &CompletionParams) -> Option<CompletionItem> {
        let mut tags = vec![];
        if self.is_deprecated() {
            tags.push(CompletionItemTag::DEPRECATED);
        }

        // Don't return a CompletionItem if it's a static and the request did not come from the file
        // of the declaration.
        if self.visibility.contains(&FunctionVisibility::Static) {
            if params.text_document_position.text_document.uri.to_string() != self.uri.to_string() {
                return None;
            }
        }

        Some(CompletionItem {
            label: self.name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            tags: Some(tags),
            detail: Some(self.type_.to_string()),
            deprecated: Some(self.is_deprecated()),
            ..Default::default()
        })
    }
}

/// Visibility of a SourcePawn function.
#[derive(Debug, PartialEq, Clone)]
pub enum FunctionVisibility {
    Public,
    Static,
    Stock,
}

/// Definition type of a SourcePawn function.
#[derive(Debug, PartialEq, Clone)]
pub enum FunctionDefinitionType {
    Forward,
    Native,
    None,
}
