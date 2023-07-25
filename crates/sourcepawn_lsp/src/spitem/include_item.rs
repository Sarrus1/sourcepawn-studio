use std::sync::Arc;

use lsp_types::{
    CompletionItem, CompletionParams, GotoDefinitionParams, Hover, HoverContents, HoverParams,
    LocationLink, MarkedString, Position, Range, Url,
};

#[derive(Debug, Clone)]
/// SPItem representation of a SourcePawn include.
pub struct IncludeItem {
    /// Name of the include.
    pub name: String,

    /// Range of the text of the include.
    pub range: Range,

    /// User visible range of the text of the include.
    pub v_range: Range,

    /// Uri of the file where the include is declared.
    pub uri: Arc<Url>,

    /// Uri of the file which the include points to.
    pub include_uri: Arc<Url>,
}

impl IncludeItem {
    /// Return `None`.
    ///
    /// # Arguments
    ///
    /// * `_params` - [CompletionParams](lsp_types::CompletionParams) of the request.
    pub(crate) fn to_completion(&self, _params: &CompletionParams) -> Option<CompletionItem> {
        None
    }

    /// Return a [Hover] from an [IncludeItem].
    ///
    /// # Arguments
    ///
    /// * `_params` - [HoverParams] of the request.
    pub(crate) fn to_hover(&self, _params: &HoverParams) -> Option<Hover> {
        Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(
                self.include_uri
                    .to_file_path()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            )),
            range: None,
        })
    }

    /// Return a [LocationLink] from an [IncludeItem].
    ///
    /// # Arguments
    ///
    /// * `_params` - [GotoDefinitionParams] of the request.
    pub(crate) fn to_definition(&self, _params: &GotoDefinitionParams) -> Option<LocationLink> {
        let zero_range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 0,
            },
        };
        Some(LocationLink {
            target_range: zero_range,
            target_uri: self.include_uri.as_ref().clone(),
            target_selection_range: zero_range,
            origin_selection_range: None,
        })
    }

    pub(crate) fn formatted_text(&self) -> String {
        format!("#include <{}>", self.name)
    }
}
