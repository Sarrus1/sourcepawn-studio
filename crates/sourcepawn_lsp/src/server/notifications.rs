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
            self.store
                .write()
                .documents
                .insert(uri.clone(), Document::new(uri, params.text_document.text));
            return Ok(());
        }

        // Don't parse the document if it has already been opened.
        // GoToDefinition request will trigger a new parse.
        if let Some(document) = self.store.read().documents.get(&uri) {
            if document.parsed {
                return Ok(());
            }
        }
        let text = params.text_document.text;
        self.store
            .write()
            .handle_open_document(&uri, text, &mut self.parser)
            .expect("Couldn't parse file");

        Ok(())
    }

    pub(super) fn did_change(
        &mut self,
        mut params: DidChangeTextDocumentParams,
    ) -> anyhow::Result<()> {
        normalize_uri(&mut params.text_document.uri);

        let uri = Arc::new(params.text_document.uri.clone());
        let Some(document) = self.store.read().get(&uri).or_else(|| {
            // If the document was not known, read its content first.
            self.store
                .write().load(uri.to_file_path().ok()?, &mut self.parser)
                .ok()?
        }) else {
            bail!("Failed to apply document edit on {}", params.text_document.uri);
        };

        let mut text = document.text().to_string();
        utils::apply_document_edit(&mut text, params.content_changes);
        self.store
            .write()
            .handle_open_document(&uri, text, &mut self.parser)?;

        self.lint_all_documents();

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
                        .reload(change.uri.to_file_path().unwrap(), &mut self.parser);
                    self.reload_diagnostics();
                }
                FileChangeType::DELETED => {
                    self.store.write().remove(&change.uri, &mut self.parser);
                    self.reload_diagnostics();
                }
                FileChangeType::CREATED => {
                    if let Ok(path) = change.uri.to_file_path() {
                        let _ = self
                            .store
                            .write()
                            .load(path.as_path().to_path_buf(), &mut self.parser);
                        self.reload_diagnostics();
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
