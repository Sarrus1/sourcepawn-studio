use std::sync::{Arc, RwLock};

use lsp_types::{Diagnostic, DiagnosticSeverity, DiagnosticTag};
use tree_sitter::{Node, QueryCursor};

use crate::{document::Document, spitem::SPItem, store::Store, utils::ts_range_to_lsp_range};

use super::parser::ERROR_QUERY;
pub(crate) mod document_diagnostics;
pub(crate) mod spcomp;

impl Store {
    /// Clear all diagnostics from the documents in the store.
    pub(super) fn clear_all_diagnostics(&mut self) {
        for document in self.documents.values_mut() {
            document.diagnostics.global_diagnostics.clear();
            document.diagnostics.sp_comp_diagnostics.clear();
        }
    }

    /// Clear all global non spcomp diagnostics from the documents in the store.
    pub(super) fn clear_all_global_diagnostics(&mut self) {
        for document in self.documents.values_mut() {
            document.diagnostics.global_diagnostics.clear();
        }
    }

    /// Lint all documents for the use of deprecated items.
    ///
    /// # Arguments
    ///
    /// * `all_items_flat` - Vector of all the [SPItems](SPItem) that are in the mainpath's scope.
    pub(super) fn get_deprecated_diagnostics(&mut self, all_items_flat: &[Arc<RwLock<SPItem>>]) {
        for item in all_items_flat.iter() {
            if let Some(description) = item.read().unwrap().description() {
                if let Some(deprecated) = description.deprecated {
                    if !&item.read().unwrap().uri().as_str().ends_with(".inc") {
                        if let Some(document) = self.documents.get_mut(&item.read().unwrap().uri())
                        {
                            document.diagnostics.local_diagnostics.push(Diagnostic {
                                range: item.read().unwrap().range(),
                                message: format!("Deprecated {:?}", deprecated),
                                severity: Some(DiagnosticSeverity::HINT),
                                tags: Some(vec![DiagnosticTag::DEPRECATED]),
                                ..Default::default()
                            });
                        }
                    }
                    if let Some(references) = item.read().unwrap().references() {
                        for reference in references.iter() {
                            if reference.uri.as_str().ends_with(".inc") {
                                continue;
                            }
                            if let Some(document) = self.documents.get_mut(&reference.uri) {
                                document.diagnostics.local_diagnostics.push(Diagnostic {
                                    range: reference.range,
                                    message: format!("Deprecated {:?}", deprecated),
                                    severity: Some(DiagnosticSeverity::HINT),
                                    tags: Some(vec![DiagnosticTag::DEPRECATED]),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Document {
    /// Capture all the syntax errors of a document and add them to its Local Diagnostics.
    /// Overrides all previous Local Diagnostics.
    ///
    /// # Arguments
    ///
    /// * `root_node` - [Root Node](tree_sitter::Node) of the document to scan.
    /// * `disable_syntax_linter` - Whether or not the syntax linter should run.
    pub(super) fn get_syntax_error_diagnostics(
        &mut self,
        root_node: Node,
        disable_syntax_linter: bool,
    ) {
        if disable_syntax_linter {
            return;
        }

        let mut cursor = QueryCursor::new();
        let matches = cursor.captures(&ERROR_QUERY, root_node, self.preprocessed_text.as_bytes());
        for (match_, _) in matches {
            for capture in match_.captures.iter() {
                self.diagnostics.local_diagnostics.push(Diagnostic {
                    range: ts_range_to_lsp_range(&capture.node.range()),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: capture.node.to_sexp(),
                    ..Default::default()
                });
            }
        }
        // TODO: Add MISSING query here once https://github.com/tree-sitter/tree-sitter/issues/606 is fixed.
    }
}
