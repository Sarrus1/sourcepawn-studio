use lsp_types::Url;
use std::{
    collections::{HashMap, HashSet},
    fs, io,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tree_sitter::Parser;
use walkdir::WalkDir;

use crate::{
    document::Document, environment::Environment, semantic_analyzer::purge_references,
    spitem::SPItem, utils::read_to_string_lossy,
};

#[derive(Clone)]
pub struct Store {
    /// Any documents the server has handled, indexed by their URL.
    pub documents: HashMap<Arc<Url>, Document>,

    pub environment: Environment,

    /// Whether this is the first parse of the documents (starting the server).
    pub first_parse: bool,

    pub watcher: Option<Arc<Mutex<notify::RecommendedWatcher>>>,
}

impl Store {
    pub fn new(current_dir: PathBuf) -> Self {
        let environment = Environment::new(Arc::new(current_dir));
        Store {
            documents: HashMap::new(),
            environment,
            first_parse: true,
            watcher: None,
        }
    }

    pub fn iter(&'_ self) -> impl Iterator<Item = Document> + '_ {
        self.documents.values().cloned()
    }

    pub fn get(&self, uri: &Url) -> Option<Document> {
        self.documents.get(uri).cloned()
    }

    pub fn remove(&mut self, uri: &Url) {
        self.documents.remove(uri);
        let uri_arc = Arc::new(uri.clone());
        let path = uri.to_file_path().unwrap();
        for document in self.documents.values_mut() {
            if document.includes.contains(uri) {
                // Consider the include to be missing.
                document
                    .missing_includes
                    .insert(path.as_path().to_str().unwrap().to_string());
            }
            document.includes.remove(uri);
            let mut sp_items = vec![];
            // Purge references to the deleted file.
            for item in document.sp_items.iter() {
                purge_references(item, &uri_arc);
                // Delete Include items.
                match &*item.read().unwrap() {
                    SPItem::Include(include_item) => {
                        if uri.ne(&*include_item.include_uri) {
                            sp_items.push(item.clone());
                        }
                    }
                    _ => sp_items.push(item.clone()),
                }
            }
            document.sp_items = sp_items;
        }
    }

    pub fn register_watcher(&mut self, watcher: notify::RecommendedWatcher) {
        self.watcher = Some(Arc::new(Mutex::new(watcher)));
    }

    pub fn load(&mut self, path: PathBuf, parser: &mut Parser) -> anyhow::Result<Option<Document>> {
        let uri = Arc::new(Url::from_file_path(path.clone()).unwrap());

        if let Some(document) = self.get(&uri) {
            return Ok(Some(document));
        }

        let data = fs::read(&path)?;
        let text = String::from_utf8_lossy(&data).into_owned();
        let document = self.handle_open_document(uri, text, parser)?;
        self.resolve_missing_includes(parser);

        Ok(Some(document))
    }

    fn resolve_missing_includes(&mut self, parser: &mut Parser) {
        let mut to_reload = HashSet::new();
        for document in self.documents.values() {
            for missing_include in document.missing_includes.iter() {
                for uri in self.documents.keys() {
                    if uri.as_str().contains(missing_include) {
                        to_reload.insert(document.uri.clone());
                    }
                }
            }
        }
        for uri in to_reload {
            if let Some(document) = self.documents.get(&uri) {
                let _ = self.handle_open_document(uri, document.text.clone(), parser);
            }
        }
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
                if self.documents.contains_key(&uri) {
                    continue;
                }
                let text = match read_to_string_lossy(uri.to_file_path().unwrap()) {
                    Ok(text) => text,
                    Err(_err) => {
                        eprintln!("Failed to read file {:?} ", uri.to_file_path().unwrap());
                        continue;
                    }
                };
                let document = Document::new(Arc::new(uri.clone()), text.clone());
                self.documents.insert(Arc::new(uri), document);
            }
        }
    }

    pub fn handle_open_document(
        &mut self,
        uri: Arc<Url>,
        text: String,
        parser: &mut Parser,
    ) -> Result<Document, io::Error> {
        let mut document = Document::new(uri, text);
        document
            .parse(self, parser)
            .expect("Couldn't parse document");
        if !self.first_parse {
            // Don't try to find references yet, all the tokens might not be referenced.
            document.find_references(self);
        }

        Ok(document)
    }

    pub fn read_unscanned_imports(&mut self, includes: &HashSet<Url>, parser: &mut Parser) {
        for include_uri in includes.iter() {
            let document = self.get(include_uri).expect("Include does not exist.");
            if document.parsed {
                continue;
            }
            let document = self
                .handle_open_document(document.uri, document.text, parser)
                .expect("Couldn't parse file");
            self.read_unscanned_imports(&document.includes, parser)
        }
    }

    pub fn find_all_references(&self) {
        for document in self.documents.values() {
            document.find_references(&self);
        }
    }

    pub fn get_all_files_in_folder(&self, folder_uri: &Url) -> Vec<Url> {
        let mut children = vec![];
        for uri in self.documents.keys() {
            if uri.as_str().contains(folder_uri.as_str()) {
                children.push((**uri).clone());
            }
        }

        children
    }
}
