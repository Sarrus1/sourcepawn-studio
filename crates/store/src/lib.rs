use ::syntax::SPItem;
use anyhow::anyhow;
use fxhash::{FxHashMap, FxHashSet};
use include::add_include;
use lsp_types::{Range, Url};
use parking_lot::RwLock;
use sourcepawn_preprocessor::{preprocessor::Macro, SourcepawnPreprocessor};
use std::{
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use tree_sitter::Parser;
use walkdir::WalkDir;

pub mod document;
pub mod environment;
pub mod include;
pub mod linter;
pub mod main_heuristic;
pub mod options;
mod parser;
pub mod semantic_analyzer;
pub mod syntax;

use crate::{
    document::{Document, Token, Walker},
    environment::Environment,
    semantic_analyzer::purge_references,
};

#[derive(Clone, Default)]
pub struct Store {
    /// Any documents the server has handled, indexed by their URL.
    pub documents: FxHashMap<Arc<Url>, Document>,

    pub environment: Environment,

    /// Whether this is the first parse of the documents (starting the server).
    pub first_parse: bool,

    pub watcher: Option<Arc<Mutex<notify::RecommendedWatcher>>>,

    pub get_all_items_time: Vec<Duration>,

    pub get_includes_time: Vec<Duration>,
}

impl Store {
    pub fn new(amxxpawn_mode: bool) -> Self {
        Store {
            first_parse: true,
            environment: Environment::new(amxxpawn_mode),
            ..Default::default()
        }
    }

    pub fn iter(&'_ self) -> impl Iterator<Item = Document> + '_ {
        self.documents.values().cloned()
    }

    pub fn get(&self, uri: &Url) -> Option<Document> {
        self.documents.get(uri).cloned()
    }

    pub fn get_text(&self, uri: &Url) -> Option<String> {
        if let Some(document) = self.documents.get(uri) {
            return Some(document.text.clone());
        }

        None
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
                match &*item.read() {
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
        let uri = Arc::new(Url::from_file_path(&path).map_err(|err| {
            anyhow!(
                "Failed to convert path to URI while loading a file: {:?}",
                err
            )
        })?);

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
        let uri = Arc::new(Url::from_file_path(&path).map_err(|err| {
            anyhow!(
                "Failed to convert path to URI while reloading a file: {:?}",
                err
            )
        })?);

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
            .filter_entry(|entry| !is_git_folder(entry.path()))
            .filter_map(|entry| entry.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            if !self.is_sourcepawn_file(entry.path()) {
                continue;
            }
            if let Ok(mut uri) = Url::from_file_path(entry.path()) {
                log::debug!("URI: {:?} path: {:?}", uri, entry.path());
                normalize_uri(&mut uri);
                if self.documents.contains_key(&uri) {
                    continue;
                }
                let Ok(text) = read_to_string_lossy(entry.path().to_path_buf()) else {
                        log::error!("Failed to read file {:?} ", entry.path());
                        continue;
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
        log::trace!("Opening file {:?}", uri);
        let prev_declarations = match self.documents.get(&(*uri).clone()) {
            Some(document) => document.declarations.clone(),
            None => FxHashMap::default(),
        };
        let mut document = Document::new(uri.clone(), text);
        self.preprocess_document(&mut document);
        self.add_sourcemod_include(&mut document);
        self.parse(&mut document, parser)
            .expect("Couldn't parse document");
        if !self.first_parse {
            // Don't try to find references yet, all the tokens might not be referenced.
            self.find_references(&(*uri).clone());
            self.sync_references(&mut document, prev_declarations);
        }
        log::trace!("Done opening file {:?}", uri);

        Ok(document)
    }

    fn add_sourcemod_include(&mut self, document: &mut Document) {
        let mut sourcemod_path = "sourcemod".to_string();
        if let Some(uri) = self.resolve_import(&mut sourcemod_path, &document.uri, false) {
            add_include(document, uri, sourcemod_path, Range::default());
        }
    }

    fn sync_references(
        &mut self,
        document: &mut Document,
        prev_declarations: FxHashMap<String, Arc<RwLock<SPItem>>>,
    ) {
        log::trace!("Syncing references for document {:?}", document.uri);
        let mut deleted_declarations = prev_declarations;
        deleted_declarations.retain(|k, _| !document.declarations.contains_key(k));
        let mut added_declarations = document.declarations.clone();
        added_declarations.retain(|k, _| !deleted_declarations.contains_key(k));
        let mut to_reload = vec![];
        for sub_doc in self.documents.values() {
            for item in added_declarations.values() {
                if sub_doc.unresolved_tokens.contains(&item.read().name()) {
                    to_reload.push(sub_doc.uri.clone());
                    break;
                }
            }
        }
        for uri_to_reload in to_reload.iter() {
            // resolve includes
            if let Some(doc_to_reload) = self.documents.get_mut(uri_to_reload) {
                for (mut missing_inc_path, range) in doc_to_reload.missing_includes.clone() {
                    // FIXME: The false in this method call may be problematic.
                    if let Some(include_uri) =
                        self.resolve_import(&mut missing_inc_path, &document.uri, false)
                    {
                        add_include(document, include_uri, missing_inc_path, range);
                    }
                }
            }
            self.find_references(uri_to_reload);
        }
        for item in deleted_declarations.values() {
            let item = item.read();
            let references = item.references();
            if let Some(references) = references {
                for ref_ in references.iter() {
                    if let Some(ref_document) = self.documents.get_mut(&ref_.uri) {
                        ref_document.unresolved_tokens.insert(item.name());
                    }
                }
            }
        }
        log::trace!("Done syncing references for document {:?}", document.uri);
    }

    pub(crate) fn preprocess_document(
        &mut self,
        document: &mut Document,
    ) -> Option<FxHashMap<String, Macro>> {
        log::trace!("Preprocessing document {:?}", document.uri);
        if !document.preprocessed_text.is_empty() || document.being_preprocessed {
            log::trace!("Skipped preprocessing document {:?}", document.uri);
            return Some(document.macros.clone());
        }
        document.being_preprocessed = true;
        let mut preprocessor = SourcepawnPreprocessor::new(document.uri.clone(), &document.text);
        let preprocessed_text = preprocessor
            .preprocess_input(
                &mut (|macros: &mut FxHashMap<String, Macro>,
                       path: String,
                       document_uri: &Url,
                       quoted: bool| {
                    self.extend_macros(macros, path, document_uri, quoted)
                }),
            )
            .unwrap_or_else(|err| {
                log::error!("{:?}", err);
                document.text.clone()
            });

        document.preprocessed_text = preprocessed_text;
        document.macros = preprocessor.macros.clone();
        document.offsets = preprocessor.offsets.clone();
        preprocessor.add_diagnostics(&mut document.diagnostics.local_diagnostics);
        document
            .macro_symbols
            .extend(preprocessor.evaluated_define_symbols.iter().map(|token| {
                Arc::new(Token {
                    text: token.text(),
                    range: token.range,
                })
            }));
        document.being_preprocessed = false;
        log::trace!("Done preprocessing document {:?}", document.uri);

        Some(preprocessor.macros)
    }

    pub(crate) fn preprocess_document_by_uri(
        &mut self,
        uri: Arc<Url>,
    ) -> Option<FxHashMap<String, Macro>> {
        log::trace!("Preprocessing document by uri {:?}", uri);
        if let Some(document) = self.documents.get(&uri) {
            // Don't reprocess the text if it has not changed.
            if !document.preprocessed_text.is_empty() || document.being_preprocessed {
                log::trace!("Skipped preprocessing document by uri {:?}", uri);
                return Some(document.macros.clone());
            }
        }
        if let Some(document) = self.documents.get_mut(&uri) {
            document.being_preprocessed = true;
        }
        if let Some(text) = self.get_text(&uri) {
            let mut preprocessor = SourcepawnPreprocessor::new(uri.clone(), &text);
            let preprocessed_text = preprocessor
                .preprocess_input(
                    &mut (|macros: &mut FxHashMap<String, Macro>,
                           path: String,
                           document_uri: &Url,
                           quoted: bool| {
                        self.extend_macros(macros, path, document_uri, quoted)
                    }),
                )
                .unwrap_or_else(|err| {
                    log::error!("{:?}", err);
                    text.clone()
                });

            if let Some(document) = self.documents.get_mut(&uri) {
                document.preprocessed_text = preprocessed_text;
                document.macros = preprocessor.macros.clone();
                preprocessor.add_diagnostics(&mut document.diagnostics.local_diagnostics);
                document
                    .macro_symbols
                    .extend(preprocessor.evaluated_define_symbols.iter().map(|token| {
                        Arc::new(Token {
                            text: token.text(),
                            range: token.range,
                        })
                    }));
            }
            return Some(preprocessor.macros);
        }
        if let Some(document) = self.documents.get_mut(&uri) {
            document.being_preprocessed = false;
        }
        log::trace!("Done preprocessing document by uri {:?}", uri);

        None
    }

    pub(crate) fn extend_macros(
        &mut self,
        macros: &mut FxHashMap<String, Macro>,
        mut include_text: String,
        document_uri: &Url,
        quoted: bool,
    ) -> anyhow::Result<()> {
        if let Some(include_uri) =
            self.resolve_import(&mut include_text, &Arc::new(document_uri.clone()), quoted)
        {
            if let Some(include_macros) = self.preprocess_document_by_uri(Arc::new(include_uri)) {
                macros.extend(include_macros);
            }
            return Ok(());
        }

        Err(anyhow!(
            "Could not resolve include \"{}\" from path.",
            include_text
        ))
    }

    pub fn parse(&mut self, document: &mut Document, parser: &mut Parser) -> anyhow::Result<()> {
        log::trace!("Parsing document {:?}", document.uri);
        let tree = parser
            .parse(&document.preprocessed_text, None)
            .ok_or(anyhow!("Failed to parse document {:?}", document.uri))?;
        let root_node = tree.root_node();
        let mut walker = Walker {
            comments: vec![],
            deprecated: vec![],
            anon_enum_counter: 0,
        };

        let mut cursor = root_node.walk();

        for mut node in root_node.children(&mut cursor) {
            let kind = node.kind();
            let _ = match kind {
                "function_declaration" | "function_definition" => {
                    document.parse_function(&node, &mut walker, None)
                }
                "global_variable_declaration" | "old_global_variable_declaration" => {
                    document.parse_variable(&mut node, None)
                }
                "preproc_include" | "preproc_tryinclude" => self.parse_include(document, &mut node),
                "enum" => document.parse_enum(&mut node, &mut walker),
                "preproc_define" => document.parse_define(&mut node, &mut walker),
                "methodmap" => document.parse_methodmap(&mut node, &mut walker),
                "typedef" => document.parse_typedef(&node, &mut walker),
                "typeset" => document.parse_typeset(&node, &mut walker),
                "preproc_macro" => document.parse_macro(&mut node, &mut walker),
                "enum_struct" => document.parse_enum_struct(&mut node, &mut walker),
                "comment" => {
                    walker.push_comment(node, &document.preprocessed_text);
                    walker.push_inline_comment(&document.sp_items);
                    Ok(())
                }
                "preproc_pragma" => walker.push_deprecated(node, &document.preprocessed_text),
                _ => continue,
            };
        }
        document.parsed = true;
        document.extract_tokens(root_node);
        document.add_macro_symbols();
        document.get_syntax_error_diagnostics(
            root_node,
            self.environment.options.disable_syntax_linter,
        );
        self.documents
            .insert(document.uri.clone(), document.clone());
        self.read_unscanned_imports(&document.includes, parser);
        log::trace!("Done parsing document {:?}", document.uri);

        Ok(())
    }

    pub(crate) fn read_unscanned_imports(
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
        let uris: Vec<Url> =
            if let Ok(Some(main_path_uri)) = self.environment.options.get_main_path_uri() {
                let mut includes = FxHashSet::default();
                includes.insert(main_path_uri.clone());
                if let Some(document) = self.documents.get(&main_path_uri) {
                    self.get_included_files(document, &mut includes);
                    includes.iter().map(|uri| (*uri).clone()).collect()
                } else {
                    self.documents.values().map(|doc| doc.uri()).collect()
                }
            } else {
                self.documents.values().map(|doc| doc.uri()).collect()
            };
        uris.iter().for_each(|uri| {
            let _ = self.find_references(uri);
        });
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

        if self.environment.amxxpawn_mode {
            f_name.ends_with(".sma") || f_name.ends_with(".inc")
        } else {
            f_name.ends_with(".sp") || f_name.ends_with(".inc")
        }
    }
}

fn is_git_folder(path: &Path) -> bool {
    let file_name = path.file_name().unwrap_or_default();
    file_name == ".git"
}

pub fn normalize_uri(uri: &mut lsp_types::Url) {
    fn fix_drive_letter(text: &str) -> Option<String> {
        if !text.is_ascii() {
            return None;
        }

        match &text[1..] {
            ":" => Some(text.to_ascii_uppercase()),
            "%3A" | "%3a" => Some(format!("{}:", text[0..1].to_ascii_uppercase())),
            _ => None,
        }
    }

    if let Some(mut segments) = uri.path_segments() {
        if let Some(mut path) = segments.next().and_then(fix_drive_letter) {
            for segment in segments {
                path.push('/');
                path.push_str(segment);
            }

            uri.set_path(&path);
        }
    }

    uri.set_fragment(None);
}

/// Read a file from its path in a lossy way. If the file non UTF-8 characters, they will be replaced
/// by a �.
///
/// # Arguments
///
/// * `path` - [Path][PathBuf] of the file.
pub fn read_to_string_lossy(path: PathBuf) -> anyhow::Result<String> {
    let mut file = File::open(path)?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;

    Ok(String::from_utf8_lossy(&buf).to_string())
}

/// Returns true if [Range] a contains [Range] b.
///
/// # Arguments
///
/// * `a` - [Range] to check against.
/// * `b` - [Range] to check against.
pub fn range_contains_range(a: &Range, b: &Range) -> bool {
    if b.start.line < a.start.line || b.end.line > a.end.line {
        return false;
    }
    if b.start.line == a.start.line && b.start.character < a.start.character {
        return false;
    }
    if b.end.line == a.end.line && b.end.character > a.end.character {
        return false;
    }

    true
}
