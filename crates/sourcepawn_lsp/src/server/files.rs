use std::{sync::Arc, time::Duration, time::Instant};

use anyhow::{anyhow, Context};
use lsp_types::{notification::ShowMessage, MessageType, ShowMessageParams, Url};

use crate::{document::Document, lsp_ext, server::InternalMessage, store::Store, Server};

mod events;
mod watching;

impl Store {
    /// Check if a document is a potential main file.
    /// Used when the mainPath was not set by the user.
    ///
    /// A document is a potential main file when it is not in an includeDirectory,
    /// if it is a .sp file and it contains `OnPluginStart(`.
    ///
    /// # Arguments
    ///
    /// * `document` - [Document] to check against.
    fn is_main_heuristic(&self, document: &Document) -> Option<Url> {
        if self.environment.amxxpawn_mode {
            return None;
        }
        let path = document.path().ok()?;
        let path = path.to_str()?;
        for include_directory in self.environment.options.includes_directories.iter() {
            if path.contains(include_directory.to_str()?) {
                return None;
            }
        }
        if document.extension().ok()? == "sp" && document.text.contains("OnPluginStart()") {
            return Some(document.uri());
        }

        None
    }

    fn find_main_with_heuristic(&self) -> Option<Url> {
        self.documents
            .values()
            .find_map(|document| self.is_main_heuristic(document))
    }
}

impl Server {
    pub(super) fn reparse_all(&mut self) -> anyhow::Result<()> {
        log::debug!("Scanning all the files.");
        self.store.write().get_all_items_time.clear();
        self.store.write().get_includes_time.clear();
        self.indexing = true;
        let _ = self.send_status(lsp_ext::ServerStatusParams {
            health: crate::lsp_ext::Health::Ok,
            quiescent: !self.indexing,
            message: None,
        });
        self.parse_directories();
        let main_uri = self.store.read().environment.options.get_main_path_uri();
        let now_parse = Instant::now();
        if let Ok(main_uri) = main_uri {
            if let Some(main_uri) = main_uri {
                log::debug!("Main path is set, parsing files.");
                self.parse_files_for_main_path(&main_uri)?;
            } else {
                if let Some(uri) = self.store.read().find_main_with_heuristic() {
                    log::debug!("Main path was not set, and was infered as {:?}", uri);
                    let path = uri.to_file_path().unwrap();
                    let mut options = self.store.read().environment.options.as_ref().clone();
                    options.main_path = path.clone();
                    self.internal_tx
                        .send(InternalMessage::SetOptions(Arc::new(options)))
                        .unwrap();
                    let _ = self
                        .client
                        .send_notification::<ShowMessage>(ShowMessageParams {
                            message: format!(
                                "MainPath was not set and was automatically infered as {}.",
                                path.file_name().unwrap().to_str().unwrap()
                            ),
                            typ: MessageType::INFO,
                        });
                    return Ok(());
                }
                log::debug!("Main path was not set, and could not be infered.");
                let _ = self
                    .client
                    .send_notification::<ShowMessage>(ShowMessageParams {
                        message: "No MainPath setting and none could be infered.".to_string(),
                        typ: MessageType::WARNING,
                    });
                self.parse_files_for_missing_main_path();
            }
        } else if main_uri.is_err() {
            log::debug!("Main path is invalid.");
            let _ = self
                .client
                .send_notification::<ShowMessage>(ShowMessageParams {
                    message: "Invalid MainPath setting.".to_string(),
                    typ: MessageType::WARNING,
                });
            self.parse_files_for_missing_main_path();
        }
        let now_analysis = Instant::now();
        self.store.write().find_all_references();
        self.store.write().first_parse = false;
        let parse_duration = now_parse.elapsed();
        let analysis_duration = now_analysis.elapsed();
        let analysis_get_items_duration = self
            .store
            .read()
            .get_all_items_time
            .iter()
            .sum::<Duration>();
        let get_includes_duration = self.store.read().get_includes_time.iter().sum::<Duration>();
        log::info!(
            r#"Scanned all the files in {:.2?}:
    - {} file(s) were scanned.
    - Parsing took {:.2?}.
    - Analysis took {:.2?}.
        - Analysis took {:.2?}.
        - Getting all items {:.2?}.
            - Getting includes took {:.2?}.
            - Cloning items took {:.2?}.
        "#,
            parse_duration,
            self.store.read().documents.len(),
            parse_duration - analysis_duration,
            analysis_duration,
            analysis_duration - analysis_get_items_duration,
            analysis_get_items_duration,
            get_includes_duration,
            analysis_get_items_duration - get_includes_duration
        );
        self.indexing = false;
        self.reload_diagnostics();
        let _ = self.send_status(lsp_ext::ServerStatusParams {
            health: crate::lsp_ext::Health::Ok,
            quiescent: !self.indexing,
            message: None,
        });

        Ok(())
    }

    fn parse_files_for_missing_main_path(&mut self) {
        let mut uris: Vec<Url> = vec![];
        for uri in self.store.read().documents.keys() {
            uris.push(uri.as_ref().clone());
        }
        for uri in uris.iter() {
            let document = self.store.read().get(uri);
            if let Some(document) = document {
                match self.store.write().handle_open_document(
                    &document.uri,
                    document.text,
                    &mut self.parser,
                ) {
                    Ok(_) => {}
                    Err(error) => {
                        log::error!("Error while parsing file: {}", error);
                    }
                }
            }
        }
    }

    fn parse_files_for_main_path(&mut self, main_uri: &Url) -> anyhow::Result<()> {
        let document = self
            .store
            .read()
            .get(main_uri)
            .context(format!("Main Path does not exist at uri {:?}", main_uri))?;
        self.store
            .write()
            .handle_open_document(&document.uri, document.text, &mut self.parser)
            .context(format!("Could not parse file at uri {:?}", main_uri))?;

        Ok(())
    }

    fn parse_directories(&mut self) {
        let directories = self
            .store
            .read()
            .environment
            .options
            .includes_directories
            .clone();
        for path in directories {
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
            self.store.write().find_documents(&path);
        }
    }

    /// Check if a [uri](Url) is know or not. If it is not, scan its parent folder and analyze all the documents that
    /// have not been scanned.
    ///
    /// # Arguments
    ///
    /// * `uri` - [Uri](Url) of the document to test for.
    pub(super) fn read_unscanned_document(&mut self, uri: Arc<Url>) -> anyhow::Result<()> {
        if self.store.read().documents.get(&uri).is_some() {
            return Ok(());
        }
        if uri.to_file_path().is_err() {
            return Err(anyhow!("Couldn't extract a path from {}", uri));
        }
        let path = uri.to_file_path().unwrap();
        let parent_dir = path.parent().unwrap().to_path_buf();
        self.store.write().find_documents(&parent_dir);
        let uris: Vec<Url> = self
            .store
            .read()
            .documents
            .keys()
            .map(|uri| uri.as_ref().clone())
            .collect();
        for uri in uris {
            if let Some(document) = self.store.read().documents.get(&uri) {
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
