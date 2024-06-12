use ide::WideEncoding;
use lsp_types::{
    CallHierarchyOptions, CallHierarchyServerCapability, ClientCapabilities, CompletionOptions,
    CompletionOptionsCompletionItem, DocumentSymbolOptions, HoverProviderCapability, MarkupKind,
    OneOf, PositionEncodingKind, ReferencesOptions, RenameOptions, SemanticTokensFullOptions,
    SemanticTokensLegend, SemanticTokensOptions, ServerCapabilities, SignatureHelpOptions,
    TextDocumentSyncCapability, TextDocumentSyncKind, WorkDoneProgressOptions,
};

use crate::{
    config::Config,
    line_index::PositionEncoding,
    lsp::{ext::negotiated_encoding, semantic_tokens},
};

pub fn server_capabilities(config: &Config) -> ServerCapabilities {
    ServerCapabilities {
        position_encoding: match negotiated_encoding(config.caps()) {
            PositionEncoding::Utf8 => Some(PositionEncodingKind::UTF8),
            PositionEncoding::Wide(wide) => match wide {
                WideEncoding::Utf16 => Some(PositionEncodingKind::UTF16),
                WideEncoding::Utf32 => Some(PositionEncodingKind::UTF32),
                _ => None,
            },
        },
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        semantic_tokens_provider: Some(
            SemanticTokensOptions {
                legend: SemanticTokensLegend {
                    token_types: semantic_tokens::SUPPORTED_TYPES.to_vec(),
                    token_modifiers: semantic_tokens::SUPPORTED_MODIFIERS.to_vec(),
                },
                full: Some(SemanticTokensFullOptions::Delta { delta: Some(true) }),
                range: Some(true),
                work_done_progress_options: Default::default(),
            }
            .into(),
        ),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(true),
            trigger_characters: Some(vec![
                "<".to_string(),
                '"'.to_string(),
                "'".to_string(),
                "/".to_string(),
                "\\".to_string(),
                ".".to_string(),
                ":".to_string(),
                " ".to_string(),
                "$".to_string(),
                "*".to_string(),
            ]),
            all_commit_characters: None,
            completion_item: completion_item(config),
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
        }),
        signature_help_provider: Some(SignatureHelpOptions {
            trigger_characters: Some(vec![",".to_string(), "(".to_string()]),
            retrigger_characters: Some(vec![",".to_string(), "(".to_string()]),
            ..Default::default()
        }),
        references_provider: Some(OneOf::Right(ReferencesOptions {
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
        })),
        rename_provider: Some(OneOf::Right(RenameOptions {
            prepare_provider: Some(false),
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
        })),
        document_symbol_provider: Some(OneOf::Right(DocumentSymbolOptions {
            label: Some("SourcePawn".to_string()),
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
        })),
        call_hierarchy_provider: Some(CallHierarchyServerCapability::Options(
            CallHierarchyOptions {
                work_done_progress_options: WorkDoneProgressOptions {
                    work_done_progress: None,
                },
            },
        )),
        ..Default::default()
    }
}

#[allow(dead_code)]
pub trait ClientCapabilitiesExt {
    fn has_definition_link_support(&self) -> bool;

    fn has_hierarchical_document_symbol_support(&self) -> bool;

    fn has_work_done_progress_support(&self) -> bool;

    fn has_hover_markdown_support(&self) -> bool;

    fn has_pull_configuration_support(&self) -> bool;

    fn has_push_configuration_support(&self) -> bool;

    fn has_file_watching_support(&self) -> bool;
}

impl ClientCapabilitiesExt for ClientCapabilities {
    fn has_definition_link_support(&self) -> bool {
        self.text_document
            .as_ref()
            .and_then(|cap| cap.definition.as_ref())
            .and_then(|cap| cap.link_support)
            == Some(true)
    }

    fn has_hierarchical_document_symbol_support(&self) -> bool {
        self.text_document
            .as_ref()
            .and_then(|cap| cap.document_symbol.as_ref())
            .and_then(|cap| cap.hierarchical_document_symbol_support)
            == Some(true)
    }

    fn has_work_done_progress_support(&self) -> bool {
        self.window.as_ref().and_then(|cap| cap.work_done_progress) == Some(true)
    }

    fn has_hover_markdown_support(&self) -> bool {
        self.text_document
            .as_ref()
            .and_then(|cap| cap.hover.as_ref())
            .and_then(|cap| cap.content_format.as_ref())
            .filter(|formats| formats.contains(&MarkupKind::Markdown))
            .is_some()
    }

    fn has_pull_configuration_support(&self) -> bool {
        self.workspace.as_ref().and_then(|cap| cap.configuration) == Some(true)
    }

    fn has_push_configuration_support(&self) -> bool {
        self.workspace
            .as_ref()
            .and_then(|cap| cap.did_change_configuration)
            .and_then(|cap| cap.dynamic_registration)
            == Some(true)
    }

    fn has_file_watching_support(&self) -> bool {
        self.workspace
            .as_ref()
            .and_then(|cap| cap.did_change_watched_files)
            .and_then(|cap| cap.dynamic_registration)
            == Some(true)
    }
}

fn completion_item(config: &Config) -> Option<CompletionOptionsCompletionItem> {
    Some(CompletionOptionsCompletionItem {
        label_details_support: Some(config.completion_label_details_support()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{
        DocumentSymbolClientCapabilities, GotoCapability, HoverClientCapabilities,
        TextDocumentClientCapabilities, WindowClientCapabilities,
    };

    #[test]
    fn test_has_definition_link_support_true() {
        let capabilities = ClientCapabilities {
            text_document: Some(TextDocumentClientCapabilities {
                definition: Some(GotoCapability {
                    link_support: Some(true),
                    ..GotoCapability::default()
                }),
                ..TextDocumentClientCapabilities::default()
            }),
            ..ClientCapabilities::default()
        };
        assert!(capabilities.has_definition_link_support());
    }

    #[test]
    fn test_has_definition_link_support_false() {
        let capabilities = ClientCapabilities::default();
        assert!(!capabilities.has_definition_link_support());
    }

    #[test]
    fn test_has_hierarchical_document_symbol_support_true() {
        let capabilities = ClientCapabilities {
            text_document: Some(TextDocumentClientCapabilities {
                document_symbol: Some(DocumentSymbolClientCapabilities {
                    hierarchical_document_symbol_support: Some(true),
                    ..DocumentSymbolClientCapabilities::default()
                }),
                ..TextDocumentClientCapabilities::default()
            }),
            ..ClientCapabilities::default()
        };
        assert!(capabilities.has_hierarchical_document_symbol_support());
    }

    #[test]
    fn test_has_hierarchical_document_symbol_support_false() {
        let capabilities = ClientCapabilities::default();
        assert!(!capabilities.has_hierarchical_document_symbol_support());
    }

    #[test]
    fn test_has_work_done_progress_support_true() {
        let capabilities = ClientCapabilities {
            window: Some(WindowClientCapabilities {
                work_done_progress: Some(true),
                ..WindowClientCapabilities::default()
            }),
            ..ClientCapabilities::default()
        };
        assert!(capabilities.has_work_done_progress_support());
    }

    #[test]
    fn test_has_work_done_progress_support_false() {
        let capabilities = ClientCapabilities::default();
        assert!(!capabilities.has_work_done_progress_support());
    }

    #[test]
    fn test_has_hover_markdown_support_true() {
        let capabilities = ClientCapabilities {
            text_document: Some(TextDocumentClientCapabilities {
                hover: Some(HoverClientCapabilities {
                    content_format: Some(vec![MarkupKind::PlainText, MarkupKind::Markdown]),
                    ..HoverClientCapabilities::default()
                }),
                ..TextDocumentClientCapabilities::default()
            }),
            ..ClientCapabilities::default()
        };
        assert!(capabilities.has_hover_markdown_support());
    }

    #[test]
    fn test_has_hover_markdown_support_false() {
        let capabilities = ClientCapabilities::default();
        assert!(!capabilities.has_hover_markdown_support());
    }
}
