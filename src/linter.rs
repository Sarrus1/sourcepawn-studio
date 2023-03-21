use std::sync::{Arc, RwLock};

use lsp_types::{Diagnostic, DiagnosticSeverity, DiagnosticTag};

use crate::{spitem::SPItem, store::Store};

pub(crate) mod document_diagnostics;
pub(crate) mod spcomp;

impl Store {
    /// Clear all diagnostics from the documents in the store.
    pub(super) fn clear_all_diagnostics(&mut self) {
        for document in self.documents.values_mut() {
            document.diagnostics.clear();
        }
    }

    /// Clear all global non spcomp diagnostics from the documents in the store.
    pub(super) fn clear_all_global_diagnostics(&mut self) {
        for document in self.documents.values_mut() {
            document.diagnostics.global_diagnostics.clear();
        }
    }

    pub(super) fn get_deprecated_diagnostics(&mut self, all_items_flat: &[Arc<RwLock<SPItem>>]) {
        for item in all_items_flat.iter() {
            if let Some(description) = item.read().unwrap().description() {
                if let Some(deprecated) = description.deprecated {
                    let document = self.documents.get_mut(&item.read().unwrap().uri()).unwrap();
                    document.diagnostics.local_diagnostics.push(Diagnostic {
                        range: item.read().unwrap().range().unwrap(),
                        message: format!("Deprecated {:?}", deprecated),
                        severity: Some(DiagnosticSeverity::HINT),
                        tags: Some(vec![DiagnosticTag::DEPRECATED]),
                        ..Default::default()
                    });
                    if let Some(references) = item.read().unwrap().references() {
                        for reference in references.iter() {
                            let document = self.documents.get_mut(&reference.uri).unwrap();
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
