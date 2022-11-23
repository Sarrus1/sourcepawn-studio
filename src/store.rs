use lazy_static::lazy_static;
use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument, Notification},
    CompletionItem, CompletionParams, CompletionResponse, DidChangeTextDocumentParams,
    DidOpenTextDocumentParams, Url,
};
use std::{collections::HashMap, fs, io, path::PathBuf, sync::Arc};
use tree_sitter::Parser;
use walkdir::WalkDir;

use crate::{document::Document, environment::Environment, server::Server, spitem::to_completion};

#[derive(Clone)]
pub struct Store {
    /// Any documents the server has handled, indexed by their URL
    pub documents: HashMap<String, Document>,
    pub environment: Environment,
}

impl Store {
    pub fn new(current_dir: PathBuf) -> Self {
        let environment = Environment::new(Arc::new(current_dir));
        Store {
            documents: HashMap::new(),
            environment,
        }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Document> + 'a {
        self.documents.values().cloned()
    }

    pub fn find_documents(&mut self, base_path: &PathBuf) {
        for entry in WalkDir::new(base_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let f_name = entry.file_name().to_string_lossy();
            if f_name.ends_with(".sp") || f_name.ends_with(".inc") {
                let uri = Url::from_file_path(entry.path()).unwrap();
                if self.documents.contains_key(&uri.to_string()) {
                    return;
                }
                let text =
                    fs::read_to_string(uri.to_file_path().unwrap()).expect("Failed to read file.");
                self.documents.insert(
                    uri.to_string(),
                    Document {
                        uri: uri.to_string(),
                        text,
                        ..Default::default()
                    },
                );
            }
        }
    }

    pub fn parse_directories(&mut self) {
        let directories = self.environment.options.includes_directories.clone();
        for path in directories {
            if !path.exists() {
                continue;
            }
            self.find_documents(&path);
        }
    }

    pub fn handle_open_document(
        &mut self,
        uri: &String,
        text: String,
        parser: &mut Parser,
    ) -> Result<(), io::Error> {
        let mut document = Document {
            uri: uri.to_string(),
            text,
            ..Default::default()
        };
        document.parse(&self.environment, parser, &self.documents);
        eprintln!("{:?}", document.includes);
        self.documents.insert(uri.to_string(), document);

        Ok(())
    }

    pub fn handle_change_document(&mut self, n: lsp_server::Notification) -> Result<(), io::Error> {
        Ok(())
    }
}
