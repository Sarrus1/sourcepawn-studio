use std::sync::Arc;

use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemTag, CompletionParams, Range, Url,
};

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
    pub description: String,

    /// Uri of the file where the function is declared.
    pub uri: Arc<Url>,

    /// Whether the function is deprecated.
    pub deprecated: bool,

    /// Full function signature.
    pub detail: String,

    /// Visibility of the function.
    pub visibility: Vec<FunctionVisibility>,

    /// Definition type of the function.
    pub definition_type: FunctionDefinitionType,
    // params: VariableItem[];
    // references: Location[];
}

/// Return a [CompletionItem](lsp_types::CompletionItem) from a [FunctionItem].
///
/// If the conditions are not appropriate (ex: asking for a static outside of its file), return None.
///
/// # Arguments
///
/// * `function_item` - [FunctionItem] to convert.
/// * `params` - [CompletionParams](lsp_types::CompletionParams) of the request.
pub(crate) fn to_completion(
    function_item: &FunctionItem,
    params: &CompletionParams,
) -> Option<CompletionItem> {
    let mut tags = vec![];
    if function_item.deprecated {
        tags.push(CompletionItemTag::DEPRECATED);
    }

    // Don't return a CompletionItem if it's a static and the request did not come from the file
    // of the declaration.
    if function_item
        .visibility
        .contains(&FunctionVisibility::Static)
    {
        if params.text_document_position.text_document.uri.to_string()
            != function_item.uri.to_string()
        {
            return None;
        }
    }

    Some(CompletionItem {
        label: function_item.name.to_string(),
        kind: Some(CompletionItemKind::FUNCTION),
        tags: Some(tags),
        ..Default::default()
    })
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
