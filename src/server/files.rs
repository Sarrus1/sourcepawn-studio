use std::{sync::Arc, time::Instant};

use lsp_types::{notification::ShowMessage, MessageType, ShowMessageParams, Url};

use crate::Server;

mod events;
mod watching;

impl Server {
    pub(super) fn reparse_all(&mut self) -> anyhow::Result<()> {
        self.indexing = true;
        self.send_status()?;
        self.parse_directories();
        let main_uri = self.store.environment.options.get_main_path_uri();
        let now = Instant::now();
        if let Ok(main_uri) = main_uri {
            if let Some(main_uri) = main_uri {
                self.parse_files_for_main_path(&main_uri);
            } else if let Some(uri) = self.store.documents.values().find_map(|document| {
                if document.extension() == "sp" && document.text.contains("OnPluginStart()") {
                    return Some(document.uri());
                }
                None
            }) {
                // Assume we found the main path.
                let path = uri.to_file_path().unwrap();
                let mut old_options = self.store.environment.options.as_ref().clone();
                old_options.main_path = path.clone();
                self.store.environment.options = Arc::new(old_options);
                self.parse_files_for_main_path(&uri);
                self.client
                    .send_notification::<ShowMessage>(ShowMessageParams {
                        message: format!(
                            "MainPath was not set and was automatically infered as {}.",
                            path.file_name().unwrap().to_str().unwrap()
                        ),
                        typ: MessageType::INFO,
                    })?;
            } else {
                // We haven't found a candidate for the main path.
                self.client
                    .send_notification::<ShowMessage>(ShowMessageParams {
                        message: "No MainPath setting and none could be infered.".to_string(),
                        typ: MessageType::WARNING,
                    })?;
                self.parse_files_for_missing_main_path();
            }
        } else if main_uri.is_err() {
            self.client
                .send_notification::<ShowMessage>(ShowMessageParams {
                    message: "Invalid MainPath setting.".to_string(),
                    typ: MessageType::WARNING,
                })?;
            self.parse_files_for_missing_main_path();
        }
        self.store.find_all_references();
        self.store.first_parse = false;
        eprintln!("Reparsed all the files in {:.2?}", now.elapsed());
        self.indexing = false;
        self.reload_diagnostics();
        self.send_status()?;

        Ok(())
    }

    fn parse_files_for_missing_main_path(&mut self) {
        let mut uris: Vec<Url> = vec![];
        for uri in self.store.documents.keys() {
            uris.push(uri.as_ref().clone());
        }
        for uri in uris.iter() {
            let document = self.store.get(uri);
            if let Some(document) = document {
                self.store
                    .handle_open_document(&document.uri, document.text, &mut self.parser)
                    .unwrap();
            }
        }
    }

    fn parse_files_for_main_path(&mut self, main_uri: &Url) {
        let document = self.store.get(main_uri).expect("Main Path does not exist.");
        self.store
            .handle_open_document(&document.uri, document.text, &mut self.parser)
            .expect("Could not parse file");
    }

    fn parse_directories(&mut self) {
        let directories = self.store.environment.options.includes_directories.clone();
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
            self.store.find_documents(&path);
        }
    }

    /// Check if a [uri](Url) is know or not. If it is not, scan its parent folder and analyze all the documents that
    /// have not been scanned.
    ///
    /// # Arguments
    ///
    /// * `uri` - [Uri](Url) of the document to test for.
    pub(super) fn read_unscanned_document(&mut self, uri: Arc<Url>) {
        if self.store.documents.get(&uri).is_some() {
            return;
        }
        let path = uri.to_file_path().unwrap();
        let parent_dir = path.parent().unwrap().to_path_buf();
        self.store.find_documents(&parent_dir);
        let uris: Vec<Url> = self
            .store
            .documents
            .keys()
            .map(|uri| uri.as_ref().clone())
            .collect();
        for uri in uris {
            let document = self.store.documents.get(&uri);
            if let Some(document) = document {
                if !document.parsed {
                    self.store
                        .handle_open_document(
                            &document.uri.clone(),
                            document.text.clone(),
                            &mut self.parser,
                        )
                        .unwrap();
                }
            }
        }
    }
}
