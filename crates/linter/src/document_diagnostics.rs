use lsp_types::Diagnostic;

use super::spcomp::SPCompDiagnostic;

#[derive(Debug, Clone, Default)]
pub struct DocumentDiagnostics {
    /// Diagnostics provided by spcomp.
    pub sp_comp_diagnostics: Vec<SPCompDiagnostic>,

    /// Diagnostics that only depend on the document they belong to, such as syntax errors.
    pub local_diagnostics: Vec<Diagnostic>,

    /// Diagnostics that depend on the includes of the document, such as unresolved items.
    pub global_diagnostics: Vec<Diagnostic>,
}

impl DocumentDiagnostics {
    /// Return a concatenation of all the [Diagnostics](lsp_types::Diagnostic).
    pub fn all(&self, disable_local_diagnostics: bool) -> Vec<Diagnostic> {
        let mut lsp_diagnostics: Vec<Diagnostic> = self
            .sp_comp_diagnostics
            .iter()
            .map(|diagnostic| diagnostic.to_lsp_diagnostic())
            .collect();
        lsp_diagnostics.extend(self.global_diagnostics.clone());
        if !disable_local_diagnostics {
            lsp_diagnostics.extend(self.local_diagnostics.clone());
        }

        lsp_diagnostics
    }
}
