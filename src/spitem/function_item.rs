use std::sync::{Arc, RwLock, Weak};

use super::Location;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemTag, CompletionParams, DocumentSymbol,
    Documentation, GotoDefinitionParams, Hover, HoverContents, HoverParams, LanguageString,
    LocationLink, MarkedString, MarkupContent, ParameterInformation, Range, SignatureInformation,
    SymbolKind, SymbolTag, Url,
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
    pub params: Vec<Arc<RwLock<SPItem>>>,

    /// Parent of the method. None if it's a first class function.
    pub parent: Option<Weak<RwLock<SPItem>>>,

    /// Children ([VariableItem](super::variable_item::VariableItem)) of this function.
    pub children: Vec<Arc<RwLock<SPItem>>>,
}

impl FunctionItem {
    fn is_deprecated(&self) -> bool {
        self.description.deprecated.is_some()
    }

    /// Return a vector of [CompletionItem](lsp_types::CompletionItem) from a [FunctionItem] and its children.
    ///
    /// If the conditions are not appropriate (ex: asking for a static outside of its file), return None.
    ///
    /// # Arguments
    ///
    /// * `params` - [CompletionParams](lsp_types::CompletionParams) of the request.
    /// * `request_method` - Whether we are requesting method completions or not.
    pub(crate) fn to_completions(
        &self,
        params: &CompletionParams,
        request_method: bool,
    ) -> Vec<CompletionItem> {
        let mut res = vec![];
        // Don't return a method if non method items are requested.
        if !request_method && self.parent.is_some() {
            return res;
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
            return res;
        }

        res.push(CompletionItem {
            label: self.name.to_string(),
            kind: Some(self.completion_kind()),
            tags: Some(tags),
            detail: Some(self.type_.to_string()),
            deprecated: Some(self.is_deprecated()),
            ..Default::default()
        });

        for child in &self.children {
            res.extend(child.read().unwrap().to_completions(params, request_method));
        }

        res
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

    /// Return a [SignatureInformation] from a [FunctionItem].
    pub(crate) fn to_signature_help(&self, parameter_count: u32) -> Option<SignatureInformation> {
        let mut parameters: Vec<ParameterInformation> = vec![];
        for param in self.params.iter() {
            let param_ = param.read().unwrap();
            parameters.push(ParameterInformation {
                label: lsp_types::ParameterLabel::Simple(param_.name()),
                documentation: match param_.description() {
                    Some(description) => Some(Documentation::String(description.text)),
                    None => None,
                },
            })
        }
        Some(SignatureInformation {
            label: self.detail.clone(),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: lsp_types::MarkupKind::Markdown,
                value: self.description.to_md(),
            })),
            parameters: Some(parameters),
            active_parameter: Some(parameter_count),
        })
    }

    /// Return a [DocumentSymbol] from a [FunctionItem].
    pub(crate) fn to_document_symbol(&self) -> Option<DocumentSymbol> {
        let mut tags = vec![];
        if self.description.deprecated.is_some() {
            tags.push(SymbolTag::DEPRECATED);
        }
        let mut kind = SymbolKind::FUNCTION;
        if let Some(parent) = &self.parent {
            match &*parent.upgrade().unwrap().read().unwrap() {
                SPItem::EnumStruct(_) => kind = SymbolKind::METHOD,
                SPItem::Methodmap(mm_item) => {
                    if mm_item.name == self.name {
                        kind = SymbolKind::CONSTRUCTOR
                    } else {
                        kind = SymbolKind::METHOD
                    }
                }
                _ => {}
            }
        }
        #[allow(deprecated)]
        Some(DocumentSymbol {
            name: self.name.to_string(),
            detail: Some(self.detail.to_string()),
            kind,
            tags: Some(tags),
            range: self.full_range,
            deprecated: None,
            selection_range: self.range,
            children: Some(
                self.children
                    .iter()
                    .filter_map(|child| child.read().unwrap().to_document_symbol())
                    .collect(),
            ),
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
            value: self.detail.to_string(),
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
