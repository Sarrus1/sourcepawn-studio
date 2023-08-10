use linter::spcomp::get_spcomp_diagnostics;
use lsp_types::{
    notification::{PublishDiagnostics, ShowMessage},
    MessageType, PublishDiagnosticsParams, ShowMessageParams,
};
use std::sync::Arc;

use super::InternalMessage;
use crate::{lsp_ext, LspClient, Server};

impl Server {
    /// Reload the diagnostics of the workspace, by running spcomp.
    pub(crate) fn reload_diagnostics(&mut self) {
        self.store.write().diagnostics.clear_all_diagnostics();

        self.lint_all_documents();

        let client = self.client.clone();
        let sender = self.internal_tx.clone();
        let store = Arc::clone(&self.store);
        if let Ok(Some(main_path_uri)) = self.store.read().environment.options.get_main_path_uri() {
            // Only reload the diagnostics if the main path is defined.
            self.pool.execute(move || {
                let _ = client.send_spcomp_status(false);
                if let Ok(diagnostics_map) = get_spcomp_diagnostics(
                    main_path_uri,
                    &store.read().environment.options.spcomp_path,
                    &store.read().environment.options.includes_directories,
                    &store.read().environment.options.linter_arguments,
                ) {
                    let _ = sender.send(InternalMessage::Diagnostics(diagnostics_map));
                } else {
                    // Failed to run spcomp.
                    let _ = client.send_notification::<ShowMessage>(ShowMessageParams {
                        message: "Failed to run spcomp.\nIs the path valid?".to_string(),
                        typ: MessageType::ERROR,
                    });
                }
                let _ = client.send_spcomp_status(true);
            });
        }
    }

    /// Lint all documents with the custom linter.
    pub fn lint_all_documents(&mut self) {
        self.store
            .write()
            .diagnostics
            .clear_all_global_diagnostics();
        let all_items_flat = self.store.read().get_all_items(true);
        // TODO: Make diagnostics an external crate to avoid having to pass the store as writable.
        self.store
            .write()
            .diagnostics
            .get_deprecated_diagnostics(&all_items_flat);
        self.publish_diagnostics();
    }

    /// Publish all the diagnostics of the store. This will override all diagnostics that have already
    /// been sent to the client.
    pub fn publish_diagnostics(&mut self) {
        eprintln!("publishing {:#?}", self.store.read().diagnostics);
        for (uri, diagnostics) in self.store.read().diagnostics.iter() {
            eprintln!("{:#?}", diagnostics);
            let _ = self
                .client
                .send_notification::<PublishDiagnostics>(PublishDiagnosticsParams {
                    uri: uri.clone(),
                    diagnostics: diagnostics
                        .all(self.store.read().environment.options.disable_syntax_linter),
                    version: None,
                });
        }
    }
}

impl LspClient {
    pub(crate) fn send_spcomp_status(&self, quiescent: bool) -> anyhow::Result<()> {
        self.send_notification::<lsp_ext::SpcompStatusNotification>(lsp_ext::SpcompStatusParams {
            quiescent,
        })?;

        Ok(())
    }
}
