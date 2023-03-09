use fxhash::FxHashMap;
use lsp_types::{
    notification::{PublishDiagnostics, ShowMessage},
    Diagnostic, MessageType, PublishDiagnosticsParams, ShowMessageParams, Url,
};

use crate::{linter::spcomp::SPCompDiagnostic, Server};

use super::InternalMessage;

impl Server {
    /// Reload the diagnostics of the workspace, by running spcomp.
    pub(crate) fn reload_diagnostics(&mut self) {
        self.store.clear_all_diagnostics();
        if let Some(main_path_uri) = self.store.environment.options.get_main_path_uri() {
            // Only reload the diagnostics if the main path is defined.
            self.spawn(move |mut server| {
                let _ = server.send_spcomp_status(false);
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
                let _ = server.send_spcomp_status(true);
            });
        }
    }

    /// Update the diagnostics of the store with the latest diagnostics, and then
    /// publish all the diagnostics of the store.
    /// This will override all diagnostics that have already been sent to the client.
    ///
    /// # Arguments
    ///
    /// * `diagnostics_map` - The latest diagnostics.
    pub(crate) fn publish_diagnostics(
        &mut self,
        diagnostics_map: FxHashMap<Url, Vec<SPCompDiagnostic>>,
    ) -> anyhow::Result<()> {
        for (uri, document) in self.store.documents.iter_mut() {
            if let Some(diagnostics) = diagnostics_map.get(uri) {
                let lsp_diagnostics: Vec<Diagnostic> = diagnostics
                    .iter()
                    .map(|diagnostic| diagnostic.to_lsp_diagnostic())
                    .collect();
                document.diagnostics = lsp_diagnostics;
            }
            self.client
                .send_notification::<PublishDiagnostics>(PublishDiagnosticsParams {
                    uri: document.uri(),
                    diagnostics: document.diagnostics.clone(),
                    version: None,
                })?;
        }

        Ok(())
    }
}
