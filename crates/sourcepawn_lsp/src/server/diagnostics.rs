use linter::spcomp::get_spcomp_diagnostics;
use lsp_types::{
    notification::{PublishDiagnostics, ShowMessage},
    MessageType, PublishDiagnosticsParams, ShowMessageParams, Url,
};
use std::sync::Arc;

use super::InternalMessage;
use crate::{lsp_ext, LspClient, Server};

impl Server {
    /// Runs [`Server::reload_project_diagnostics()`](#method.reload_project_diagnostics) by getting the main path
    /// from the uri provided.
    ///
    /// # Arguments
    /// * `uri` - [Url] of a file in the project to reload the diagnostics of.
    pub(crate) fn reload_diagnostics(&mut self, uri: Url) {
        let Some(file_id) = self.store.read().path_interner.get(&uri) else {
            return;
        };
        let Some(main_node) = self.store.read().projects.find_root_from_id(file_id) else {
            return;
        };
        let main_path_uri = self
            .store
            .read()
            .path_interner
            .lookup(main_node.file_id)
            .clone();
        self.reload_project_diagnostics(main_path_uri);
    }

    /// Reload the diagnostics of a project, by running spcomp and the server's linter.
    ///
    /// # Arguments
    /// * `main_path_uri` - [Url] of the main file of the project.
    pub(crate) fn reload_project_diagnostics(&mut self, main_path_uri: Url) {
        self.store.write().diagnostics.clear_all_diagnostics();

        self.lint_project(&main_path_uri);

        let client = self.client.clone();
        let sender = self.internal_tx.clone();
        let store = Arc::clone(&self.store);
        // Only reload the diagnostics if the main path is defined.
        self.pool.execute(move || {
            let _ = client.send_spcomp_status(false);
            let options = store.read().environment.options.clone();
            let result = get_spcomp_diagnostics(
                main_path_uri,
                &options.spcomp_path,
                &options.includes_directories,
                &options.linter_arguments,
            );
            match result {
                Ok(diagnostics_map) => {
                    let _ = sender.send(InternalMessage::Diagnostics(diagnostics_map));
                }
                Err(err) => {
                    // Failed to run spcomp.
                    let _ = client.send_notification::<ShowMessage>(ShowMessageParams {
                        message: format!("Failed to run spcomp.\n{:?}", err),
                        typ: MessageType::ERROR,
                    });
                }
            }
            let _ = client.send_spcomp_status(true);
        });
    }

    /// Lint all documents in the project with the custom linter.
    pub fn lint_project(&mut self, uri: &Url) {
        let Some(file_id) = self.store.read().path_interner.get(uri) else {
            return;
        };
        self.store
            .write()
            .diagnostics
            .clear_all_global_diagnostics();
        let all_items_flat = self.store.read().get_all_items(&file_id, true);
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
        log::debug!("publishing {:#?}", self.store.read().diagnostics);
        for (uri, diagnostics) in self.store.read().diagnostics.iter() {
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
