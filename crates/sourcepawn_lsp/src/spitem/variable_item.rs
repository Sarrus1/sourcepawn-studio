use std::sync::{Arc, RwLock, Weak};

use super::Location;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionItemTag,
    CompletionParams, DocumentSymbol, GotoDefinitionParams, Hover, HoverContents, HoverParams,
    LanguageString, LocationLink, MarkedString, Range, SymbolKind, SymbolTag, Url,
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

    /// User visible range of the name of the variable.
    pub v_range: Range,

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
    pub parent: Option<Weak<RwLock<SPItem>>>,
}

impl VariableItem {
    /// Return a [CompletionItem](lsp_types::CompletionItem) from a [VariableItem].
    ///
    /// If the conditions are not appropriate (ex: asking for a static outside of its scope), return None.
    ///
    /// # Arguments
    ///
    /// * `params` - [CompletionParams](lsp_types::CompletionParams) of the request.
    /// * `request_method` - Whether we are requesting method completions or not.
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
            Some(parent) => match &*parent.upgrade().unwrap().read().unwrap() {
                SPItem::Function(parent) => {
                    if self.uri.to_string()
                        != params.text_document_position.text_document.uri.to_string()
                    {
                        return None;
                    }
                    if !range_contains_pos(
                        parent.v_full_range,
                        params.text_document_position.position,
                    ) {
                        return None;
                    }
                    Some(CompletionItem {
                        label: self.name.to_string(),
                        kind: Some(CompletionItemKind::VARIABLE),
                        tags: Some(tags),
                        label_details: Some(CompletionItemLabelDetails {
                            detail: Some(self.type_.clone()),
                            description: Some("local".to_string()),
                        }),
                        data: Some(serde_json::Value::String(self.key())),
                        ..Default::default()
                    })
                }
                SPItem::EnumStruct(parent) => {
                    // Don't return a field if non method items are requested.
                    if !request_method {
                        return None;
                    }
                    Some(CompletionItem {
                        label: self.name.to_string(),
                        kind: Some(CompletionItemKind::FIELD),
                        tags: Some(tags),
                        label_details: Some(CompletionItemLabelDetails {
                            detail: Some(self.type_.clone()),
                            description: Some(format!("{}::{}", parent.name, self.name)),
                        }),
                        data: Some(serde_json::Value::String(self.key())),
                        ..Default::default()
                    })
                }
                _ => {
                    log::warn!("Unhandled case in variable_item to_completion.");
                    None
                }
            },
            None => Some(CompletionItem {
                label: self.name.to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                tags: Some(tags),
                label_details: Some(CompletionItemLabelDetails {
                    detail: Some(self.type_.clone()),
                    description: Some("global".to_string()),
                }),
                data: Some(serde_json::Value::String(self.key())),
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
        let mut contents = vec![MarkedString::LanguageString(LanguageString {
            language: "sourcepawn".to_string(),
            value: self.formatted_text(),
        })];
        if let Some(md_text) = self.description.to_md() {
            contents.push(MarkedString::String(md_text))
        }
        Some(Hover {
            contents: HoverContents::Array(contents),
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
            target_range: self.v_range,
            target_uri: self.uri.as_ref().clone(),
            target_selection_range: self.v_range,
            origin_selection_range: None,
        })
    }

    /// Return a [DocumentSymbol] from a [VariableItem].
    pub(crate) fn to_document_symbol(&self) -> Option<DocumentSymbol> {
        let mut tags = vec![];
        if self.description.deprecated.is_some() {
            tags.push(SymbolTag::DEPRECATED);
        }
        let mut kind = SymbolKind::VARIABLE;
        if let Some(parent) = &self.parent {
            if let SPItem::EnumStruct(_) = &*parent.upgrade().unwrap().read().unwrap() {
                kind = SymbolKind::FIELD;
            }
        }
        #[allow(deprecated)]
        Some(DocumentSymbol {
            name: self.name.to_string(),
            detail: Some(self.detail.to_string()),
            kind,
            tags: Some(tags),
            range: self.v_range,
            deprecated: None,
            selection_range: self.v_range,
            children: None,
        })
    }

    /// Return a key to be used as a unique identifier in a map containing all the items.
    pub(crate) fn key(&self) -> String {
        match &self.parent {
            Some(parent) => format!(
                "{}-{}",
                parent.upgrade().unwrap().read().unwrap().key(),
                self.name
            ),
            None => self.name.clone(),
        }
    }

    /// Formatted representation of a [VariableItem].
    ///
    /// # Exemple
    ///
    /// `int foo;`
    pub(crate) fn formatted_text(&self) -> String {
        let mut visibility = "".to_string();
        for vis in self.visibility.iter() {
            visibility.push_str(vis.to_string());
            visibility.push(' ');
        }
        let mut storage_class = "".to_string();
        for sto in self.storage_class.iter() {
            storage_class.push_str(sto.to_string());
            storage_class.push(' ');
        }
        let prefix = format!("{}{}", visibility, storage_class);
        format!("{} {} {};", prefix.trim(), self.type_, self.name)
            .trim()
            .to_string()
    }
}

/// Visibility of a SourcePawn variable.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum VariableVisibility {
    Public,
    Stock,
}

impl VariableVisibility {
    fn to_string(&self) -> &str {
        match self {
            Self::Public => "public",
            Self::Stock => "stock",
        }
    }
}

/// Storage class of a SourcePawn variable.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum VariableStorageClass {
    Const,
    Static,
}

impl VariableStorageClass {
    fn to_string(&self) -> &str {
        match self {
            Self::Const => "const",
            Self::Static => "static",
        }
    }
}
