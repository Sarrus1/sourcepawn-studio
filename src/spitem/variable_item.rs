use std::sync::Arc;

use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemTag, CompletionParams, Range, Url,
};

#[derive(Debug, Clone)]
/// SPItem representation of a SourcePawn variable.
pub struct VariableItem {
    /// Name of the variable.
    pub name: String,

    /// Type of the variable.
    pub type_: String,

    /// Range of the name of the variable.
    pub range: Range,

    /// Description of the variable.
    pub description: String,

    /// Uri of the file where the variable is declared.
    pub uri: Arc<Url>,

    /// Whether the variable is deprecated.
    pub deprecated: bool,

    /// Full variable signature.
    pub detail: String,

    /// Visibility of the variable.
    pub visibility: Vec<VariableVisibility>,
    // references: Location[];
    // pub parent
}

/// Return a [CompletionItem](lsp_types::CompletionItem) from a [VariableItem].
///
/// If the conditions are not appropriate (ex: asking for a static outside of its scope), return None.
///
/// # Arguments
///
/// * `variable_item` - [VariableItem] to convert.
/// * `params` - [CompletionParams](lsp_types::CompletionParams) of the request.
pub(crate) fn to_completion(
    variable_item: &VariableItem,
    params: &CompletionParams,
) -> Option<CompletionItem> {
    eprintln!("{}", variable_item.name);
    let mut tags = vec![];
    if variable_item.deprecated {
        tags.push(CompletionItemTag::DEPRECATED);
    }

    Some(CompletionItem {
        label: variable_item.name.to_string(),
        kind: Some(CompletionItemKind::VARIABLE),
        tags: Some(tags),
        ..Default::default()
    })
}

/// Visibility of a SourcePawn variable.
#[derive(Debug, PartialEq, Clone)]
pub enum VariableVisibility {
    Public,
    Static,
    Stock,
}
