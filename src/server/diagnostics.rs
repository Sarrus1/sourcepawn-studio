use fxhash::FxHashMap;
use lsp_types::{
    notification::{PublishDiagnostics, ShowMessage},
    Diagnostic, MessageType, PublishDiagnosticsParams, ShowMessageParams, Url,
};

use crate::{linter::SPCompDiagnostic, Server};

use super::InternalMessage;

impl Server {
    pub(crate) fn reload_diagnostics(&mut self) {
        if let Some(main_path_uri) = self.store.environment.options.get_main_path_uri() {
            // Only reload the diagnostics if the main path is defined.
            self.spawn(move |mut server| {
                if let Ok(diagnostics_map) = server.store.get_spcomp_diagnostics(main_path_uri) {
                    server
                        .internal_tx
                        .send(InternalMessage::Diagnostics(diagnostics_map))
                        .unwrap();
                } else {
                    // Failed to run spcomp.
                    let _ = server
                        .client
                        .send_notification::<ShowMessage>(ShowMessageParams {
                            message: "Failed to run spcomp.\nIs the path valid?".to_string(),
                            typ: MessageType::ERROR,
                        });
                }
            });
        }
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
