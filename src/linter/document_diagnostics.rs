use lsp_types::Diagnostic;

use super::spcomp::SPCompDiagnostic;

#[derive(Debug, Clone, Default)]
pub struct DocumentDiagnostics {
    /// Diagnostics provided by spcomp.
    pub(super) sp_comp_diagnostics: Vec<SPCompDiagnostic>,

    /// Diagnostics that only depend on the document they belong to, such as syntax errors.
    pub(super) local_diagnostics: Vec<Diagnostic>,

    /// Diagnostics that depend on the includes of the document, such as unresolved items.
    pub(super) global_diagnostics: Vec<Diagnostic>,
}

impl DocumentDiagnostics {
    /// Clear all the diagnostics.
    pub(super) fn clear(&mut self) {
        self.sp_comp_diagnostics.clear();
        self.local_diagnostics.clear();
        self.global_diagnostics.clear();
    }

    /// Return a concatenation of all the [Diagnostics](lsp_types::Diagnostic).
    pub(crate) fn all(&self) -> Vec<Diagnostic> {
        let mut lsp_diagnostics: Vec<Diagnostic> = self
            .sp_comp_diagnostics
            .iter()
            .map(|diagnostic| diagnostic.to_lsp_diagnostic())
            .collect();
        lsp_diagnostics.extend(self.global_diagnostics.clone());
        lsp_diagnostics.extend(self.local_diagnostics.clone());

        lsp_diagnostics
    }
}
