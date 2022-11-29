use std::sync::Arc;

use lsp_types::{CompletionItem, CompletionItemKind, CompletionParams, Location, Range, Url};

use crate::providers::hover::description::Description;

#[derive(Debug, Clone)]
/// SPItem representation of a SourcePawn enum.
pub struct EnumItem {
    /// Name of the enum.
    pub name: String,

    /// Range of the name of the enum.
    pub range: Range,

    /// Range of the whole enum, including its block.
    pub full_range: Range,

    /// Description of the enum.
    pub description: Description,

    /// Uri of the file where the enum is declared.
    pub uri: Arc<Url>,

    /// References to this enum.
    pub references: Vec<Location>,
}

impl EnumItem {
    /// Return a [CompletionItem](lsp_types::CompletionItem) from an [EnumItem].
    ///
    /// # Arguments
    ///
    /// * `_params` - [CompletionParams](lsp_types::CompletionParams) of the request.
    pub(crate) fn to_completion(&self, _params: &CompletionParams) -> Option<CompletionItem> {
        if self.name.contains("#") {
            return None;
        }

        Some(CompletionItem {
            label: self.name.to_string(),
            kind: Some(CompletionItemKind::ENUM),
            detail: self.detail(),
            ..Default::default()
        })
    }

    fn detail(&self) -> Option<String> {
        match self.uri.to_file_path() {
            Ok(path) => match path.as_path().file_name() {
                Some(file_name) => match file_name.to_str() {
                    Some(file_name) => Some(file_name.to_string()),
                    None => None,
                },
                None => None,
            },
            Err(_) => None,
        }
    }
}
