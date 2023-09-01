use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionParams,
    DocumentSymbol, GotoDefinitionParams, Hover, HoverContents, HoverParams, LanguageString,
    LocationLink, MarkedString, Range, SymbolKind, SymbolTag, Url,
};
use parking_lot::RwLock;
use std::sync::Arc;

use crate::description::Description;
use crate::{uri_to_file_name, FileId, Reference, SPItem};

#[derive(Debug, Clone)]
/// SPItem representation of a SourcePawn methodmap.
pub struct MethodmapItem {
    /// Name of the methodmap.
    pub name: String,

    /// Parent of the methodmap.
    pub parent: Option<Arc<RwLock<SPItem>>>,

    /// Temporary parent of the methodmap.
    pub tmp_parent: Option<String>,

    /// Range of the name of the methodmap.
    pub range: Range,

    /// User visible range of the name of the methodmap.
    pub v_range: Range,

    /// Range of the whole methodmap, including its value.
    pub full_range: Range,

    /// User visible range of the whole methodmap, including its value.
    pub v_full_range: Range,

    /// Description of the methodmap.
    pub description: Description,

    /// Uri of the file where the methodmap is declared.
    pub uri: Arc<Url>,

    /// [FileId](FileId) of the file where the methodmap is declared.
    pub file_id: FileId,

    /// References to this methodmap.
    pub references: Vec<Reference>,

    /// Children ([FunctionItem](super::function_item::FunctionItem),
    /// [PropertyItem](super::property_item::PropertyItem)) of this methodmap.
    pub children: Vec<Arc<RwLock<SPItem>>>,
}

impl MethodmapItem {
    /// Return a vector of [CompletionItem](lsp_types::CompletionItem) from a [MethodmapItem] and its children.
    ///
    /// # Arguments
    ///
    /// * `params` - [CompletionParams](lsp_types::CompletionParams) of the request.
    /// * `request_method` - Whether we are requesting method completions or not.
    pub fn to_completions(
        &self,
        params: &CompletionParams,
        request_method: bool,
    ) -> Vec<CompletionItem> {
        let mut res = vec![];

        res.push(CompletionItem {
            label: self.name.to_string(),
            kind: Some(CompletionItemKind::CLASS),
            label_details: Some(CompletionItemLabelDetails {
                detail: None,
                description: if *self.uri != params.text_document_position.text_document.uri {
                    uri_to_file_name(&self.uri)
                } else {
                    None
                },
            }),
            data: Some(serde_json::Value::String(self.completion_data())),
            ..Default::default()
        });

        for child in &self.children {
            res.extend(child.read().to_completions(params, request_method));
        }

        res
    }

    /// Return a [Hover] from an [MethodmapItem].
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

    /// Return a [LocationLink] from a [MethodmapItem].
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

    /// Return a [DocumentSymbol] from a [MethodmapItem].
    pub fn to_document_symbol(&self) -> Option<DocumentSymbol> {
        let mut tags = vec![];
        if self.description.deprecated.is_some() {
            tags.push(SymbolTag::DEPRECATED);
        }
        #[allow(deprecated)]
        Some(DocumentSymbol {
            name: self.name.to_string(),
            detail: None,
            kind: SymbolKind::CLASS,
            tags: Some(tags),
            range: self.v_full_range,
            deprecated: None,
            selection_range: self.v_range,
            children: Some(
                self.children
                    .iter()
                    .filter_map(|child| child.read().to_document_symbol())
                    .collect(),
            ),
        })
    }

    /// Return a key to be used as a unique identifier in a map containing all the items.
    pub fn key(&self) -> String {
        self.name.clone()
    }

    pub fn completion_data(&self) -> String {
        format!("{}${}", self.key(), self.file_id)
    }

    /// Returns the constructor of the methodmap if it exists.
    pub fn ctor(&self) -> Option<Arc<RwLock<SPItem>>> {
        self.children
            .iter()
            .find(|child| {
                if let SPItem::Function(method_item) = &*child.read() {
                    return method_item.is_ctor();
                }
                false
            })
            .cloned()
    }

    /// Formatted representation of the methodmap.
    ///
    /// # Exemple
    ///
    /// `methodmap Foo < Bar`
    pub(crate) fn formatted_text(&self) -> String {
        let mut suffix = "".to_string();
        if self.parent.is_some() {
            suffix = format!(" < {}", self.parent.as_ref().unwrap().read().name());
        }
        format!("methodmap {}{}", self.name, suffix)
            .trim()
            .to_string()
    }
}
