use document_diagnostics::DocumentDiagnostics;
use fxhash::FxHashMap;
use lazy_static::lazy_static;
use lsp_types::{Diagnostic, DiagnosticSeverity, Url};
use spcomp::SPCompDiagnostic;
use std::sync::Arc;
use syntax::utils::ts_range_to_lsp_range;
use tree_sitter::{Node, Query, QueryCursor};

pub mod document_diagnostics;
pub mod spcomp;

lazy_static! {
    pub(crate) static ref ERROR_QUERY: Query =
        Query::new(&tree_sitter_sourcepawn::language(), "(ERROR) @error")
            .expect("Could not build error query.");
}

#[derive(Debug, Default, Clone)]
pub struct DiagnosticsManager {
    diagnostics: FxHashMap<Url, DocumentDiagnostics>,
}

impl DiagnosticsManager {
    pub fn iter(&self) -> impl Iterator<Item = (&Url, &DocumentDiagnostics)> {
        self.diagnostics.iter()
    }
}

impl DiagnosticsManager {
    pub fn get(&self, uri: &Arc<Url>) -> Option<&DocumentDiagnostics> {
        self.diagnostics.get(uri)
    }

    pub fn get_mut(&mut self, uri: &Url) -> &mut DocumentDiagnostics {
        self.diagnostics.entry(uri.clone()).or_default()
    }

    pub fn reset(&mut self, uri: &Url) {
        self.diagnostics.remove(uri);
        self.diagnostics
            .insert(uri.clone(), DocumentDiagnostics::default());
    }

    /// Clear all diagnostics from the documents in the store.
    pub fn clear_all_diagnostics(&mut self) {
        for diagnostics in self.diagnostics.values_mut() {
            diagnostics.global_diagnostics.clear();
            diagnostics.sp_comp_diagnostics.clear();
        }
    }

    /// Clear all global non spcomp diagnostics from the documents in the store.
    pub fn clear_all_global_diagnostics(&mut self) {
        for diagnostics in self.diagnostics.values_mut() {
            diagnostics.global_diagnostics.clear();
        }
    }

    /// Capture all the syntax errors of a document and add them to its Local Diagnostics.
    /// Overrides all previous Local Diagnostics.
    ///
    /// # Arguments
    ///
    /// * `root_node` - [Root Node](tree_sitter::Node) of the document to scan.
    /// * `disable_syntax_linter` - Whether or not the syntax linter should run.
    pub fn get_syntax_error_diagnostics(
        &mut self,
        uri: &Arc<Url>,
        source: &str,
        root_node: Node,
        disable_syntax_linter: bool,
    ) {
        if disable_syntax_linter {
            return;
        }
        let diagnostics = self.get_mut(uri);
        let mut cursor = QueryCursor::new();
        let matches = cursor.captures(&ERROR_QUERY, root_node, source.as_bytes());
        for (match_, _) in matches {
            for capture in match_.captures.iter() {
                diagnostics.local_diagnostics.push(Diagnostic {
                    range: ts_range_to_lsp_range(&capture.node.range()),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: capture.node.to_sexp(),
                    ..Default::default()
                });
            }
        }
        // TODO: Add MISSING query here once https://github.com/tree-sitter/tree-sitter/issues/606 is fixed.
    }

    /// Ingest a map of spcomp_diganostics into the [Store].
    pub fn ingest_spcomp_diagnostics(
        &mut self,
        spcomp_diagnostics_map: FxHashMap<Url, Vec<SPCompDiagnostic>>,
    ) {
        for (uri, spcomp_diagnostics) in spcomp_diagnostics_map.iter() {
            self.get_mut(uri).sp_comp_diagnostics = (*spcomp_diagnostics).clone();
        }
    }
}
