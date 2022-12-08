use std::sync::{Arc, Mutex};

use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, Hover, HoverContents, HoverParams,
    LanguageString, Location, MarkedString, Range, Url,
};

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
    pub parent: Arc<Mutex<SPItem>>,

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
            detail: Some(self.parent.lock().unwrap().name().clone()),
            ..Default::default()
        })
    }

    /// Return a [Hover] from an [EnumItem].
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

    /// Formatted representation of the enum member.
    ///
    /// # Exemple
    ///
    /// `Plugin_Continue`
    fn formatted_text(&self) -> MarkedString {
        let mut value = "".to_string();
        match &*self.parent.lock().unwrap() {
            SPItem::Enum(parent) => {
                if parent.name.contains("#") {
                    value = self.name.clone()
                } else {
                    value = format!("{}::{}", parent.name, self.name);
                }
            }
            _ => {}
        }
        MarkedString::LanguageString(LanguageString {
            language: "sourcepawn".to_string(),
            value: value,
        })
    }
}
