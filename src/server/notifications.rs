use crate::{capabilities::ClientCapabilitiesExt, dispatch, utils};
use std::sync::Arc;

use crate::Server;
use lsp_server::Notification;
use lsp_types::{
    notification::{
        DidChangeConfiguration, DidChangeTextDocument, DidChangeWatchedFiles, DidOpenTextDocument,
    },
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
    DidOpenTextDocumentParams, FileChangeType,
};

impl Server {
    pub(super) fn did_open(&mut self, mut params: DidOpenTextDocumentParams) -> anyhow::Result<()> {
        if !self.config_pulled {
            return Ok(());
        }
        utils::normalize_uri(&mut params.text_document.uri);
        let uri = Arc::new(params.text_document.uri);

        // Don't parse the document if it has already been opened.
        // GoToDefinition request will trigger a new parse.
        let document = self.store.documents.get(&uri);
        if let Some(document) = document {
            if document.parsed {
                return Ok(());
            }
        }
        let text = params.text_document.text;
        self.store
            .handle_open_document(&uri, text, &mut self.parser)
            .expect("Couldn't parse file");

        Ok(())
    }

    pub(super) fn did_change(
        &mut self,
        mut params: DidChangeTextDocumentParams,
    ) -> anyhow::Result<()> {
        utils::normalize_uri(&mut params.text_document.uri);

        let uri = Arc::new(params.text_document.uri.clone());

        match self.store.get(&uri) {
            Some(old_document) => {
                let mut text = old_document.text().to_string();
                utils::apply_document_edit(&mut text, params.content_changes);
                self.store
                    .handle_open_document(&uri, text, &mut self.parser)?;
            }
            None => match uri.to_file_path() {
                Ok(path) => {
                    self.store.load(path, &mut self.parser)?;
                }
                Err(_) => return Ok(()),
            },
        };

        self.lint_all_documents();

        Ok(())
    }

    pub(super) fn did_change_watched_files(
        &mut self,
        params: DidChangeWatchedFilesParams,
    ) -> anyhow::Result<()> {
        for mut change in params.changes {
            utils::normalize_uri(&mut change.uri);
            match change.typ {
                FileChangeType::CHANGED => {
                    let _ = self
                        .store
                        .reload(change.uri.to_file_path().unwrap(), &mut self.parser);
                    self.reload_diagnostics();
                }
                FileChangeType::DELETED => {
                    self.store.remove(&change.uri, &mut self.parser);
                    self.reload_diagnostics();
                }
                FileChangeType::CREATED => {
                    if let Ok(path) = change.uri.to_file_path() {
                        let _ = self
                            .store
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
        if self
            .store
            .environment
            .client_capabilities
            .has_pull_configuration_support()
        {
            self.spawn(move |server| {
                let _ = server.pull_config();
            });
        } else {
            let options = self.fork().parse_options(params.settings)?;
            self.store.environment.options = Arc::new(options);
            self.config_pulled = true;
            self.reparse_all()?;
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
