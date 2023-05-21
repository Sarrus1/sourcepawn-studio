use std::sync::{Arc, RwLock};

use super::{parameter::Parameter, Location};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemTag, CompletionParams, CompletionTextEdit,
    DocumentSymbol, GotoDefinitionParams, Hover, HoverContents, HoverParams, InsertTextFormat,
    LanguageString, LocationLink, MarkedString, Range, SymbolKind, SymbolTag, TextEdit, Url,
};

use crate::providers::hover::description::Description;

#[derive(Debug, Clone)]
/// SPItem representation of a SourcePawn typedef/functag, which can be converted to a
/// [CompletionItem](lsp_types::CompletionItem), [Location](lsp_types::Location), etc.
pub struct TypedefItem {
    /// Name of the typedef.
    pub name: String,

    /// Return type of the typedef.
    pub type_: String,

    /// Range of the name of the typedef.
    pub range: Range,

    /// User visible range of the name of the typedef.
    pub v_range: Range,

    /// Range of the whole typedef.
    pub full_range: Range,

    /// User visible range of the whole typedef.
    pub v_full_range: Range,

    /// Description of the typedef.
    pub description: Description,

    /// Uri of the file where the typedef is declared.
    pub uri: Arc<Url>,

    /// Full typedef text.
    pub detail: String,

    /// References to this typedef.
    pub references: Vec<Location>,

    /// Parameters of the typedef.
    pub params: Vec<Arc<RwLock<Parameter>>>,
}

impl TypedefItem {
    fn is_deprecated(&self) -> bool {
        self.description.deprecated.is_some()
    }

    /// Return a [CompletionItem](lsp_types::CompletionItem) from a [TypedefItem].
    ///
    /// # Arguments
    ///
    /// * `_params` - [CompletionParams](lsp_types::CompletionParams) of the request.
    pub(crate) fn to_completion(&self, _params: &CompletionParams) -> Option<CompletionItem> {
        let mut tags = vec![];
        if self.is_deprecated() {
            tags.push(CompletionItemTag::DEPRECATED);
        }

        Some(CompletionItem {
            label: self.name.to_string(),
            kind: Some(CompletionItemKind::INTERFACE),
            tags: Some(tags),
            detail: Some(self.type_.to_string()),
            deprecated: Some(self.is_deprecated()),
            ..Default::default()
        })
    }

    /// Return a [Hover] from a [TypedefItem].
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

    /// Return a [LocationLink] from a [TypedefItem].
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

    /// Return a [DocumentSymbol] from a [TypedefItem].
    pub(crate) fn to_document_symbol(&self) -> Option<DocumentSymbol> {
        let mut tags = vec![];
        if self.description.deprecated.is_some() {
            tags.push(SymbolTag::DEPRECATED);
        }
        #[allow(deprecated)]
        Some(DocumentSymbol {
            name: self.name.to_string(),
            detail: Some(self.detail.to_string()),
            kind: SymbolKind::INTERFACE,
            tags: Some(tags),
            range: self.v_full_range,
            deprecated: None,
            selection_range: self.v_range,
            children: None,
        })
    }

    /// Return a snippet [CompletionItem] from a [TypedefItem] for a callback completion.
    ///
    /// # Arguments
    ///
    /// * `range` - [Range] of the "$" that will be replaced.
    pub(crate) fn to_snippet_completion(&self, range: Range) -> Option<CompletionItem> {
        let mut tags = vec![];
        if self.is_deprecated() {
            tags.push(CompletionItemTag::DEPRECATED);
        }

        let mut snippet_text = format!("{} ${{1:name}}(", self.type_);
        for (i, parameter) in self.params.iter().enumerate() {
            let parameter = parameter.read().unwrap();
            if parameter.is_const {
                snippet_text.push_str("const ");
            }
            if let Some(type_) = &parameter.type_ {
                snippet_text.push_str(&type_.name);
                if type_.is_pointer {
                    snippet_text.push('&')
                }
                for dimension in type_.dimensions.iter() {
                    snippet_text.push_str(dimension);
                }
                snippet_text.push(' ');
            }
            snippet_text.push_str(&format!("${{{}:{}}}", i + 2, parameter.name));
            for dimension in parameter.dimensions.iter() {
                snippet_text.push_str(dimension);
            }
            if i < self.params.len() - 1 {
                snippet_text.push_str(", ");
            }
        }
        snippet_text.push_str(")\n{\n\t$0\n}");

        Some(CompletionItem {
            label: self.name.to_string(),
            filter_text: Some(format!("${}", self.name)),
            kind: Some(CompletionItemKind::FUNCTION),
            tags: Some(tags),
            detail: Some(self.type_.to_string()),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                range,
                new_text: snippet_text,
            })),
            deprecated: Some(self.is_deprecated()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        })
    }

    /// Return a key to be used as a unique identifier in a map containing all the items.
    pub(crate) fn key(&self) -> String {
        self.name.clone()
    }

    /// Formatted representation of a [TypedefItem].
    ///
    /// # Exemple
    ///
    /// `typedef ConCmd = function Action (int client, int args);`
    fn formatted_text(&self) -> MarkedString {
        MarkedString::LanguageString(LanguageString {
            language: "sourcepawn".to_string(),
            value: self.detail.to_string(),
        })
    }
}
