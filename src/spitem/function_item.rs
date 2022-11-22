use lsp_types::{CompletionItem, CompletionItemKind, CompletionItemTag, Range};

#[derive(Debug)]
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
    pub uri_string: String,

    /// Whether the function is deprecated.
    pub deprecated: bool,

    /// Full function signature.
    pub detail: String,

    /// Visibility of the function.
    pub visibility: Vec<FunctionVisibility>,
    // params: VariableItem[];
    // references: Location[];
}

pub(crate) fn to_completion(function_item: &FunctionItem) -> CompletionItem {
    let mut tags = vec![];
    if function_item.deprecated {
        tags.push(CompletionItemTag::DEPRECATED);
    }
    CompletionItem {
        label: function_item.name.to_string(),
        kind: Some(CompletionItemKind::FUNCTION),
        tags: Some(tags),
        ..Default::default()
    }
}

#[derive(Debug)]
pub enum FunctionVisibility {
    Public,
    Static,
    Stock,
}
