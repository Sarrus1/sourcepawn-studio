use anyhow::{anyhow, Context};
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

        // 1. Get all the mainpaths.
        // 2. Parse all the files for each mainpath.
        // 3. Resolve names in each file by getting the mainpath from the graph.
        let projects = self.store.write().load_projects_graph();
        for node in projects.find_roots() {
            // FIXME: Do we need to set the main path here?
            let _ = self.parse_project(node.file_id);
            // self.store.write().find_all_references();
        }
        self.store.write().projects = projects;

        // let main_uri = self.store.read().environment.options.get_main_path_uri();
        // if let Ok(main_uri) = main_uri {
        //     if let Some(main_uri) = main_uri {
        //         log::debug!("Main path is set, parsing files.");
        //         self.parse_files_for_main_path(&main_uri)?;
        //     } else {
        //         if let Some(uri) = self.store.read().find_main_with_heuristic() {
        //             log::debug!("Main path was not set, and was infered as {:?}", uri);
        //             let path = uri.to_file_path().unwrap();
        //             let mut options = self.store.read().environment.options.as_ref().clone();
        //             options.main_path = path.clone();
        //             self.internal_tx
        //                 .send(InternalMessage::SetOptions(Arc::new(options)))
        //                 .unwrap();
        //             let _ = self
        //                 .client
        //                 .send_notification::<ShowMessage>(ShowMessageParams {
        //                     message: format!(
        //                         "MainPath was not set and was automatically infered as {}.",
        //                         path.file_name().unwrap().to_str().unwrap()
        //                     ),
        //                     typ: MessageType::INFO,
        //                 });
        //             return Ok(());
        //         }
        //         log::debug!("Main path was not set, and could not be infered.");
        //         let _ = self
        //             .client
        //             .send_notification::<ShowMessage>(ShowMessageParams {
        //                 message: "No MainPath setting and none could be infered.".to_string(),
        //                 typ: MessageType::WARNING,
        //             });
        //         self.parse_files_for_missing_main_path();
        //     }
        // } else if main_uri.is_err() {
        //     log::debug!("Main path is invalid.");
        //     let _ = self
        //         .client
        //         .send_notification::<ShowMessage>(ShowMessageParams {
        //             message: "Invalid MainPath setting.".to_string(),
        //             typ: MessageType::WARNING,
        //         });
        //     self.parse_files_for_missing_main_path();
        // }
        //     let now_analysis = Instant::now();
        //     self.store.write().find_all_references();
        //     self.store.write().first_parse = false;
        //     let parse_duration = now_parse.elapsed();
        //     let analysis_duration = now_analysis.elapsed();
        //     log::info!(
        //         r#"Scanned all the files in {:.2?}:
        // - {} file(s) were scanned.
        // - Parsing took {:.2?}.
        // - Analysis took {:.2?}.
        //     "#,
        //         parse_duration,
        //         self.store.read().documents.len(),
        //         parse_duration - analysis_duration,
        //         analysis_duration,
        //     );
        self.indexing = false;
        let store = self.store.read();
        let file_ids: Vec<FileId> = store
            .projects
            .find_roots()
            .iter()
            .map(|node| node.file_id)
            .collect();
        drop(store);
        for file_id in file_ids {
            let uri = self.store.read().path_interner.lookup(file_id).clone();
            self.reload_diagnostics(&uri);
        }
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
            return Err(anyhow!("Couldn't extract a path from {}", uri));
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
