use std::sync::Arc;

use lsp_types::{CompletionItem, CompletionItemKind, CompletionParams, Location, Range, Url};

use crate::providers::hover::description::Description;

use super::SPItem;

#[derive(Debug, Clone)]
/// SPItem representation of a SourcePawn enum member.
pub struct EnumMemberItem {
    /// Name of the enum member.
    pub name: String,

    /// Range of the name of the enum member.
    pub range: Range,

    /// Parent of the enum member.
    pub parent: Arc<SPItem>,

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
        Some(CompletionItem {
            label: self.name.clone(),
            kind: Some(CompletionItemKind::ENUM_MEMBER),
            detail: Some(self.parent.name().clone()),
            ..Default::default()
        })
    }
}
