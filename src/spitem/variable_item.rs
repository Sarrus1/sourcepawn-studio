use std::sync::{Arc, RwLock};

use super::Location;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemTag, CompletionParams, DocumentSymbol,
    GotoDefinitionParams, Hover, HoverContents, HoverParams, LanguageString, LocationLink,
    MarkedString, Range, SymbolKind, SymbolTag, Url,
};

use crate::{providers::hover::description::Description, utils::range_contains_pos};

use super::SPItem;

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
    pub description: Description,

    /// Uri of the file where the variable is declared.
    pub uri: Arc<Url>,

    /// Full variable signature.
    pub detail: String,

    /// Visibility of the variable.
    pub visibility: Vec<VariableVisibility>,

    /// Visibility of the variable.
    pub storage_class: Vec<VariableStorageClass>,

    /// References to this variable.
    pub references: Vec<Location>,

    /// Parent of this variable, if it is not global.
    pub parent: Option<Arc<RwLock<SPItem>>>,
}

impl VariableItem {
    /// Return a [CompletionItem](lsp_types::CompletionItem) from a [VariableItem].
    ///
    /// If the conditions are not appropriate (ex: asking for a static outside of its scope), return None.
    ///
    /// # Arguments
    ///
    /// * `params` - [CompletionParams](lsp_types::CompletionParams) of the request.
    pub(crate) fn to_completion(
        &self,
        params: &CompletionParams,
        request_method: bool,
    ) -> Option<CompletionItem> {
        let mut tags = vec![];
        if self.description.deprecated.is_some() {
            tags.push(CompletionItemTag::DEPRECATED);
        }

        match &self.parent {
            Some(parent) => match &*parent.read().unwrap() {
                SPItem::Function(parent) => {
                    if self.uri.to_string()
                        != params.text_document_position.text_document.uri.to_string()
                    {
                        return None;
                    }
                    if !range_contains_pos(
                        parent.full_range,
                        params.text_document_position.position,
                    ) {
                        return None;
                    }
                    Some(CompletionItem {
                        label: self.name.to_string(),
                        kind: Some(CompletionItemKind::VARIABLE),
                        tags: Some(tags),
                        ..Default::default()
                    })
                }
                SPItem::EnumStruct(_) => {
                    // Don't return a field if non method items are requested.
                    if !request_method {
                        return None;
                    }
                    Some(CompletionItem {
                        label: self.name.to_string(),
                        kind: Some(CompletionItemKind::FIELD),
                        tags: Some(tags),
                        ..Default::default()
                    })
                }
                _ => {
                    eprintln!("Unhandled case in variable_item to_completion.");
                    None
                }
            },
            None => Some(CompletionItem {
                label: self.name.to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                tags: Some(tags),
                ..Default::default()
            }),
        }
    }

    /// Return a [Hover] from a [VariableItem].
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

    /// Return a [LocationLink] from a [VariableItem].
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

    /// Return a [DocumentSymbol] from a [VariableItem].
    pub(crate) fn to_document_symbol(&self) -> Option<DocumentSymbol> {
        let mut tags = vec![];
        if self.description.deprecated.is_some() {
            tags.push(SymbolTag::DEPRECATED);
        }
        Some(DocumentSymbol {
            name: self.name.to_string(),
            detail: Some(self.detail.to_string()),
            kind: SymbolKind::VARIABLE,
            tags: Some(tags),
            range: self.range,
            deprecated: None,
            selection_range: self.range,
            children: None,
        })
    }

    /// Formatted representation of a [VariableItem].
    ///
    /// # Exemple
    ///
    /// `int foo;`
    fn formatted_text(&self) -> MarkedString {
        MarkedString::LanguageString(LanguageString {
            language: "sourcepawn".to_string(),
            value: format!("{} {};", self.type_, self.name),
        })
    }
}

/// Visibility of a SourcePawn variable.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum VariableVisibility {
    Public,
    Stock,
}

/// Storage class of a SourcePawn variable.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum VariableStorageClass {
    Const,
    Static,
}
