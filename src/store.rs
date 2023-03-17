use fxhash::{FxHashMap, FxHashSet};
use lsp_types::Url;
use std::{
    fs, io,
    path::{Path, PathBuf},
    str::Utf8Error,
    sync::{Arc, Mutex, RwLock},
};
use tree_sitter::Parser;
use walkdir::WalkDir;

use crate::{
    document::{Document, Token, Walker},
    environment::Environment,
    parser::include_parser::{add_include, resolve_import},
    semantic_analyzer::purge_references,
    spitem::SPItem,
    utils::read_to_string_lossy,
};

#[derive(Clone)]
pub struct Store {
    /// Any documents the server has handled, indexed by their URL.
    pub documents: FxHashMap<Arc<Url>, Document>,

    pub environment: Environment,

    /// Whether this is the first parse of the documents (starting the server).
    pub first_parse: bool,

    pub watcher: Option<Arc<Mutex<notify::RecommendedWatcher>>>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            documents: FxHashMap::default(),
            environment: Environment::new(),
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

    pub fn remove(&mut self, uri: &Url, parser: &mut Parser) {
        // Open the document as empty to delete the references.
        let _ = self.handle_open_document(&Arc::new((*uri).clone()), "".to_string(), parser);
        self.documents.remove(uri);
        let uri_arc = Arc::new(uri.clone());
        for document in self.documents.values_mut() {
            if let Some(include) = document.includes.get(uri) {
                // Consider the include to be missing.
                document
                    .missing_includes
                    .insert(include.text.clone(), include.range);
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

        if !self.is_sourcepawn_file(&path) {
            return Ok(None);
        }

        let data = fs::read(&path)?;
        let text = String::from_utf8_lossy(&data).into_owned();
        let document = self.handle_open_document(&uri, text, parser)?;
        self.resolve_missing_includes(parser);

        Ok(Some(document))
    }

    pub fn reload(
        &mut self,
        path: PathBuf,
        parser: &mut Parser,
    ) -> anyhow::Result<Option<Document>> {
        let uri = Arc::new(Url::from_file_path(path.clone()).unwrap());

        if !self.is_sourcepawn_file(&path) {
            return Ok(None);
        }

        let data = fs::read(&path)?;
        let text = String::from_utf8_lossy(&data).into_owned();
        let document = self.handle_open_document(&uri, text, parser)?;
        self.resolve_missing_includes(parser);

        Ok(Some(document))
    }

    fn resolve_missing_includes(&mut self, parser: &mut Parser) {
        let mut to_reload = FxHashSet::default();
        for document in self.documents.values() {
            for missing_include in document.missing_includes.keys() {
                for uri in self.documents.keys() {
                    if uri.as_str().contains(missing_include) {
                        to_reload.insert(document.uri.clone());
                    }
                }
            }
        }
        for uri in to_reload {
            if let Some(document) = self.documents.get(&uri) {
                let _ = self.handle_open_document(&uri, document.text.clone(), parser);
            }
        }
    }

    pub fn find_documents(&mut self, base_path: &PathBuf) {
        for entry in WalkDir::new(base_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if self.is_sourcepawn_file(entry.path()) {
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
        uri: &Arc<Url>,
        text: String,
        parser: &mut Parser,
    ) -> Result<Document, io::Error> {
        let prev_declarations = match self.documents.get(&(*uri).clone()) {
            Some(document) => document.declarations.clone(),
            None => FxHashMap::default(),
        };
        let mut document = Document::new(uri.clone(), text);
        self.parse(&mut document, parser)
            .expect("Couldn't parse document");
        if !self.first_parse {
            // Don't try to find references yet, all the tokens might not be referenced.
            self.find_references(&(*uri).clone());
            self.sync_references(&mut document, prev_declarations);
        }

        Ok(document)
    }

    fn sync_references(
        &mut self,
        document: &mut Document,
        prev_declarations: FxHashMap<String, Arc<RwLock<SPItem>>>,
    ) {
        let mut deleted_declarations = prev_declarations;
        deleted_declarations.retain(|k, _| !document.declarations.contains_key(k));
        let mut added_declarations = document.declarations.clone();
        added_declarations.retain(|k, _| !deleted_declarations.contains_key(k));
        let mut to_reload = vec![];
        for sub_doc in self.documents.values() {
            for item in added_declarations.values() {
                if sub_doc
                    .unresolved_tokens
                    .contains(&item.read().unwrap().name())
                {
                    to_reload.push(sub_doc.uri.clone());
                    break;
                }
            }
        }
        let documents_keys = self.documents.keys().cloned().collect();
        for uri_to_reload in to_reload.iter() {
            // resolve includes
            if let Some(doc_to_reload) = self.documents.get_mut(uri_to_reload) {
                for (mut missing_inc_path, range) in doc_to_reload.missing_includes.clone() {
                    if let Some(include_uri) = resolve_import(
                        &self.environment.options.includes_directories,
                        &mut missing_inc_path,
                        &documents_keys,
                        &document.uri,
                    ) {
                        add_include(document, include_uri, missing_inc_path, range);
                    }
                }
            }
            self.find_references(uri_to_reload);
        }
        for item in deleted_declarations.values() {
            let item = item.read().unwrap();
            let references = item.references();
            if let Some(references) = references {
                for ref_ in references.iter() {
                    if let Some(ref_document) = self.documents.get_mut(&ref_.uri) {
                        ref_document.unresolved_tokens.insert(item.name());
                    }
                }
            }
        }
    }

    pub fn parse(&mut self, document: &mut Document, parser: &mut Parser) -> Result<(), Utf8Error> {
        let tree = parser.parse(&document.text, None).unwrap();
        let root_node = tree.root_node();
        let mut walker = Walker {
            comments: vec![],
            deprecated: vec![],
            anon_enum_counter: 0,
        };

        let mut cursor = root_node.walk();

        for mut node in root_node.children(&mut cursor) {
            let kind = node.kind();
            match kind {
                "function_declaration" | "function_definition" => {
                    document.parse_function(&node, &mut walker, None)?;
                }
                "global_variable_declaration" | "old_global_variable_declaration" => {
                    document.parse_variable(&mut node, None)?;
                }
                "preproc_include" | "preproc_tryinclude" => {
                    self.parse_include(document, &mut node)?;
                }
                "enum" => {
                    document.parse_enum(&mut node, &mut walker)?;
                }
                "preproc_define" => {
                    document.parse_define(&mut node, &mut walker)?;
                }
                "methodmap" => {
                    document.parse_methodmap(&mut node, &mut walker)?;
                }
                "typedef" => document.parse_typedef(&node, &mut walker)?,
                "typeset" => document.parse_typeset(&node, &mut walker)?,
                "preproc_macro" => {}
                "enum_struct" => document.parse_enum_struct(&mut node, &mut walker)?,
                "comment" => {
                    walker.push_comment(node, &document.text);
                }
                "preproc_pragma" => walker.push_deprecated(node, &document.text),
                _ => {
                    continue;
                }
            }
        }
        document.parsed = true;
        document.extract_tokens(root_node);
        self.documents
            .insert(document.uri.clone(), document.clone());
        self.read_unscanned_imports(&document.includes, parser);

        Ok(())
    }

    pub fn read_unscanned_imports(
        &mut self,
        includes: &FxHashMap<Url, Token>,
        parser: &mut Parser,
    ) {
        for include_uri in includes.keys() {
            let document = self.get(include_uri).expect("Include does not exist.");
            if document.parsed {
                continue;
            }
            let document = self
                .handle_open_document(&document.uri, document.text, parser)
                .expect("Couldn't parse file");
            self.read_unscanned_imports(&document.includes, parser)
        }
    }

    pub fn find_all_references(&mut self) {
        let uris: Vec<Arc<Url>> = self.documents.keys().map(|uri| (*uri).clone()).collect();
        for uri in uris {
            self.find_references(&uri);
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

    fn is_sourcepawn_file(&self, path: &Path) -> bool {
        let f_name = path.file_name().unwrap_or_default().to_string_lossy();
        f_name.ends_with(".sp") || f_name.ends_with(".inc")
    }
}
