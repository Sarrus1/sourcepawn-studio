use fxhash::FxHashMap;
use lsp_types::{notification::PublishDiagnostics, Diagnostic, PublishDiagnosticsParams, Url};

use crate::{linter::SPCompDiagnostic, Server};

use super::InternalMessage;

impl Server {
    pub(crate) fn reload_diagnostics(&mut self) {
        self.spawn(move |mut server| {
            let diagnostics_map = server.store.get_spcomp_diagnostics(
                server
                    .store
                    .environment
                    .options
                    .get_main_path_uri()
                    .unwrap(),
            );
            server
                .internal_tx
                .send(InternalMessage::Diagnostics(diagnostics_map))
                .unwrap();
        });
    }

    pub(crate) fn publish_diagnostics(
        &mut self,
        diagnostics_map: FxHashMap<Url, Vec<SPCompDiagnostic>>,
    ) -> anyhow::Result<()> {
        for (uri, diagnostics) in diagnostics_map {
            let lsp_diagnostics: Vec<Diagnostic> = diagnostics
                .iter()
                .map(|diagnostic| diagnostic.to_lsp_diagnostic())
                .collect();
            if let Some(document) = self.store.documents.get_mut(&uri) {
                document.diagnostics = lsp_diagnostics.clone();
            }
            self.client
                .send_notification::<PublishDiagnostics>(PublishDiagnosticsParams {
                    uri,
                    diagnostics: lsp_diagnostics,
                    version: None,
                })?;
        }

        Ok(())
    }
}
