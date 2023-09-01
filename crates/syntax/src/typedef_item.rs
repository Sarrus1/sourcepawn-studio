use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionItemTag,
    CompletionParams, CompletionTextEdit, DocumentSymbol, GotoDefinitionParams, Hover,
    HoverContents, HoverParams, InsertTextFormat, LanguageString, LocationLink, MarkedString,
    Range, SymbolKind, SymbolTag, TextEdit, Url,
};
use parking_lot::RwLock;
use std::sync::{Arc, Weak};

use crate::description::Description;
use crate::parameter::Parameter;
use crate::{uri_to_file_name, FileId, Reference, SPItem};

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

    /// [FileId](FileId) of the file where the typedef is declared.
    pub file_id: FileId,

    /// Full typedef text.
    pub detail: String,

    /// References to this typedef.
    pub references: Vec<Reference>,

    /// Parameters of the typedef.
    pub params: Vec<Arc<RwLock<Parameter>>>,

    /// Parent of the typedef, if it belongs to a typeset.
    pub parent: Option<Weak<RwLock<SPItem>>>,
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
    pub fn to_completion(&self, params: &CompletionParams) -> Option<CompletionItem> {
        let mut tags = vec![];
        if self.is_deprecated() {
            tags.push(CompletionItemTag::DEPRECATED);
        }

        Some(CompletionItem {
            label: self.name.to_string(),
            kind: Some(CompletionItemKind::INTERFACE),
            tags: Some(tags),
            label_details: Some(CompletionItemLabelDetails {
                detail: None,
                description: if *self.uri != params.text_document_position.text_document.uri {
                    uri_to_file_name(&self.uri)
                } else {
                    None
                },
            }),
            deprecated: Some(self.is_deprecated()),
            data: Some(serde_json::Value::String(self.completion_data())),
            ..Default::default()
        })
    }

    /// Return a [Hover] from a [TypedefItem].
    ///
    /// # Arguments
    ///
    /// * `_params` - [HoverParams] of the request.
    pub fn to_hover(&self, _params: &HoverParams) -> Option<Hover> {
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

    /// Return a [LocationLink] from a [TypedefItem].
    ///
    /// # Arguments
    ///
    /// * `_params` - [GotoDefinitionParams] of the request.
    pub fn to_definition(&self, _params: &GotoDefinitionParams) -> Option<LocationLink> {
        Some(LocationLink {
            target_range: self.v_range,
            target_uri: self.uri.as_ref().clone(),
            target_selection_range: self.v_range,
            origin_selection_range: None,
        })
    }

    /// Return a [DocumentSymbol] from a [TypedefItem].
    pub fn to_document_symbol(&self) -> Option<DocumentSymbol> {
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
    pub fn to_snippet_completion(&self, range: Range) -> Option<CompletionItem> {
        let mut tags = vec![];
        if self.is_deprecated() {
            tags.push(CompletionItemTag::DEPRECATED);
        }

        let mut snippet_text = format!("{} ${{1:name}}(", self.type_);
        for (i, parameter) in self.params.iter().enumerate() {
            let parameter = parameter.read();
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
            label_details: Some(CompletionItemLabelDetails {
                detail: Some(self.type_.to_string()),
                description: uri_to_file_name(&self.uri),
            }),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                range,
                new_text: snippet_text,
            })),
            deprecated: Some(self.is_deprecated()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            data: Some(serde_json::Value::String(self.completion_data())),
            ..Default::default()
        })
    }

    /// Return a key to be used as a unique identifier in a map containing all the items.
    pub fn key(&self) -> String {
        match &self.parent {
            Some(parent) => format!("{}-{}", parent.upgrade().unwrap().read().key(), self.name),
            None => self.name.clone(),
        }
    }

    pub fn completion_data(&self) -> String {
        format!("{}${}", self.key(), self.file_id)
    }

    /// Formatted representation of a [TypedefItem].
    ///
    /// # Exemple
    ///
    /// `typedef ConCmd = function Action (int client, int args);`
    pub(crate) fn formatted_text(&self) -> String {
        self.detail.to_string()
    }
}
