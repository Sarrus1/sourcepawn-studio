use std::sync::Arc;

use anyhow::bail;
use lsp_server::Notification;
use lsp_types::{
    notification::{
        DidChangeConfiguration, DidChangeTextDocument, DidChangeWatchedFiles, DidOpenTextDocument,
    },
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
    DidOpenTextDocumentParams, FileChangeType,
};
use store::{document::Document, normalize_uri};

use crate::{capabilities::ClientCapabilitiesExt, dispatch, utils, Server};

impl Server {
    pub(super) fn did_open(&mut self, mut params: DidOpenTextDocumentParams) -> anyhow::Result<()> {
        normalize_uri(&mut params.text_document.uri);
        let uri = Arc::new(params.text_document.uri);

        if !self.config_pulled {
            log::trace!("File {:?} was opened before the config was pulled.", uri);
            let file_id = self
                .store
                .write()
                .path_interner
                .intern(uri.as_ref().clone());
            self.store.write().documents.insert(
                file_id,
                Document::new(uri, file_id, params.text_document.text),
            );
            return Ok(());
        }

        // Don't parse the document if it has already been opened.
        // GoToDefinition request will trigger a new parse.
        if let Some(document) = self.store.read().get_from_uri(&uri) {
            if document.parsed {
                return Ok(());
            }
        }
        let text = params.text_document.text;
        self.store
            .write()
            .handle_open_document(&uri, text)
            .expect("Couldn't parse file");

        // In the first parse, it is expected that includes are missing.
        if !self.store.read().first_parse {
            self.store.write().resolve_missing_includes();
        }

        Ok(())
    }

    pub(super) fn did_change(
        &mut self,
        mut params: DidChangeTextDocumentParams,
    ) -> anyhow::Result<()> {
        normalize_uri(&mut params.text_document.uri);

        let uri = Arc::new(params.text_document.uri.clone());
        let Some(document) = self.store.read().get_cloned_from_uri(&uri).or_else(|| {
            // If the document was not known, read its content first.
            self.store.write().load(uri.to_file_path().ok()?).ok()?
        }) else {
            bail!(
                "Failed to apply document edit on {}",
                params.text_document.uri
            );
        };

        let mut text = document.text().to_string();
        utils::apply_document_edit(&mut text, params.content_changes);
        self.store.write().handle_open_document(&uri, text)?;

        self.lint_project(&params.text_document.uri);

        Ok(())
    }

    pub(super) fn did_change_watched_files(
        &mut self,
        params: DidChangeWatchedFilesParams,
    ) -> anyhow::Result<()> {
        for mut change in params.changes {
            normalize_uri(&mut change.uri);
            match change.typ {
                FileChangeType::CHANGED => {
                    let _ = self
                        .store
                        .write()
                        .reload(change.uri.to_file_path().unwrap());
                    self.reload_diagnostics(change.uri);
                }
                FileChangeType::DELETED => {
                    self.store.write().remove(&change.uri);
                    self.reload_diagnostics(change.uri);
                }
                FileChangeType::CREATED => {
                    if let Ok(path) = change.uri.to_file_path() {
                        let _ = self.store.write().load(path.as_path().to_path_buf());
                        self.reload_diagnostics(change.uri);
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub(super) fn did_change_configuration(
        &mut self,
        params: DidChangeConfigurationParams,
    ) -> anyhow::Result<()> {
        if self.client_capabilities.has_pull_configuration_support() {
            self.pull_config();
        } else {
            let options = self.client.parse_options(params.settings)?;
            self.store.write().environment.options = Arc::new(options);
            self.config_pulled = true;
            let _ = self.reparse_all();
        }

        Ok(())
    }

    pub(super) fn handle_notification(&mut self, notification: Notification) -> anyhow::Result<()> {
        dispatch::NotificationDispatcher::new(notification)
            .on::<DidOpenTextDocument, _>(|params| self.did_open(params))?
            .on::<DidChangeTextDocument, _>(|params| self.did_change(params))?
            .on::<DidChangeConfiguration, _>(|params| self.did_change_configuration(params))?
            .on::<DidChangeWatchedFiles, _>(|params| self.did_change_watched_files(params))?
            .default();

        Ok(())
    }
}
