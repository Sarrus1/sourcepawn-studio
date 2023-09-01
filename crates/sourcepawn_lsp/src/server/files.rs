use anyhow::{anyhow, bail, Context};
use lsp_types::{notification::ShowMessage, MessageType, ShowMessageParams, Url};
use std::sync::Arc;
use syntax::FileId;

use crate::{lsp_ext, Server};

mod events;
mod watching;

impl Server {
    pub(super) fn reparse_all(&mut self) -> anyhow::Result<()> {
        log::debug!("Scanning all the files.");
        self.indexing = true;
        let _ = self.send_status(lsp_ext::ServerStatusParams {
            health: crate::lsp_ext::Health::Ok,
            quiescent: !self.indexing,
            message: None,
        });
        self.parse_directories();

        let projects = self.store.write().load_projects_graph();
        for node in projects.find_roots() {
            let _ = self.parse_project(node.file_id);
        }
        self.store.write().projects = projects;

        self.indexing = false;
        let _ = self.send_status(lsp_ext::ServerStatusParams {
            health: crate::lsp_ext::Health::Ok,
            quiescent: !self.indexing,
            message: None,
        });

        Ok(())
    }

    fn parse_project(&mut self, main_id: FileId) -> anyhow::Result<()> {
        let document = self
            .store
            .read()
            .get_cloned(&main_id)
            .context(format!("Main Path does not exist for id {:?}", main_id))?;
        self.store
            .write()
            .handle_open_document(&document.uri, document.text, &mut self.parser)
            .context(format!("Could not parse file at id {:?}", main_id))?;

        Ok(())
    }

    fn parse_directories(&mut self) {
        let store = self.store.read();
        let folders = store.folders();
        drop(store);
        for path in folders {
            if !path.exists() {
                self.client
                    .send_notification::<ShowMessage>(ShowMessageParams {
                        message: format!(
                            "Invalid IncludeDirectory path: {}",
                            path.to_str().unwrap_or_default()
                        ),
                        typ: MessageType::WARNING,
                    })
                    .unwrap_or_default();
                continue;
            }
            self.store.write().discover_documents(&path);
        }
    }

    /// Check if a [uri](Url) is know or not. If it is not, scan its parent folder and analyze all the documents that
    /// have not been scanned.
    ///
    /// # Arguments
    ///
    /// * `uri` - [Uri](Url) of the document to test for.
    pub(super) fn read_unscanned_document(&mut self, uri: Arc<Url>) -> anyhow::Result<()> {
        let file_id = self.store.read().path_interner.get(&uri).ok_or(anyhow!(
            "Couldn't get a file id from the path interner for {}",
            uri
        ))?;
        if self.store.read().documents.get(&file_id).is_some() {
            return Ok(());
        }
        if uri.to_file_path().is_err() {
            bail!("Couldn't extract a path from {}", uri);
        }
        let path = uri.to_file_path().unwrap();
        let parent_dir = path.parent().unwrap().to_path_buf();
        self.store.write().discover_documents(&parent_dir);
        for file_id in self.store.read().documents.keys() {
            if let Some(document) = self.store.read().documents.get(file_id) {
                if !document.parsed {
                    self.store
                        .write()
                        .handle_open_document(
                            &document.uri.clone(),
                            document.text.clone(),
                            &mut self.parser,
                        )
                        .unwrap();
                }
            }
        }

        Ok(())
    }
}
