use ::syntax::SPItem;
use ::vfs::FileId;
use anyhow::anyhow;
use fxhash::{FxHashMap, FxHashSet};
use linter::DiagnosticsManager;
use lsp_types::{Range, Url};
use parking_lot::RwLock;
use parser::Parser;
use semantic_analyzer::{purge_references, Token};
use std::{
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use walkdir::WalkDir;

pub mod document;
pub mod environment;
pub mod include;
pub mod options;
mod semantics;
pub mod syntax;

use crate::{document::Document, environment::Environment};

fn spawn_parser() -> tree_sitter::Parser {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_sourcepawn::language())
        .expect("Error loading SourcePawn grammar");
    parser
}

#[derive(Debug, Clone, Default)]
pub struct Store {
    /// Any documents the server has handled, indexed by their URL.
    pub documents: FxHashMap<FileId, Document>,

    pub environment: Environment,

    /// Whether this is the first parse of the documents (starting the server).
    pub first_parse: bool,

    pub watcher: Option<Arc<Mutex<notify::RecommendedWatcher>>>,

    pub diagnostics: DiagnosticsManager,

    pub folders: Vec<PathBuf>,
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

    /// Returns the [MainPath](PathBuf) of a project given a the [FileId](FileId) of a file in the project.
    pub fn get_project_main_path_from_id(&self, _file_id: &FileId) -> Option<PathBuf> {
        // if let Some(node) = self.projects.find_root_from_id(*file_id) {
        //     return self.vfs.lookup(node.file_id).to_file_path().ok();
        // }

        None
    }

    pub fn contains_uri(&self, _uri: &Url) -> bool {
        // let Some(file_id) = self.vfs.get(uri) else {
        //     return false;
        // };
        // self.documents.contains_key(&file_id)
        false
    }

    pub fn get_from_uri(&self, _uri: &Url) -> Option<&Document> {
        // self.documents.get(&self.vfs.get(uri)?)
        None
    }

    pub fn get_cloned_from_uri(&self, _uri: &Url) -> Option<Document> {
        // self.documents.get(&self.vfs.get(uri)?).cloned()
        None
    }

    pub fn get_cloned(&self, file_id: &FileId) -> Option<Document> {
        self.documents.get(file_id).cloned()
    }

    pub fn get_text(&self, file_id: &FileId) -> Option<String> {
        if let Some(document) = self.documents.get(file_id) {
            return Some(document.text.clone());
        }

        None
    }

    pub fn folders(&self) -> Vec<PathBuf> {
        let mut res = self.folders.clone();
        res.extend(
            self.environment
                .options
                .includes_directories
                .iter()
                .cloned(),
        );

        res
    }

    pub fn remove(&mut self, uri: &Url) {
        // let Some(file_id) = self.vfs.get(uri) else {
        //     return;
        // };
        let file_id = FileId(0);
        // Open the document as empty to delete the references.
        let _ = self.handle_open_document(&Arc::new((*uri).clone()), "".to_string());
        self.documents.remove(&file_id);
        for document in self.documents.values_mut() {
            if let Some(include) = document.includes.get(&file_id) {
                // Consider the include to be missing.
                document
                    .missing_includes
                    .insert(include.text.clone(), include.range);
            }
            document.includes.remove(&file_id);
            let mut sp_items = vec![];
            // Purge references to the deleted file.
            for item in document.sp_items.iter() {
                purge_references(item, file_id);
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

        // self.remove_file_from_projects(&file_id);
    }

    pub fn register_watcher(&mut self, watcher: notify::RecommendedWatcher) {
        self.watcher = Some(Arc::new(Mutex::new(watcher)));
    }

    pub fn load(&mut self, path: PathBuf) -> anyhow::Result<Option<Document>> {
        let mut uri = Url::from_file_path(&path).map_err(|err| {
            anyhow!(
                "Failed to convert path to URI while loading a file: {:?}",
                err
            )
        })?;
        normalize_uri(&mut uri);
        if !self.is_sourcepawn_file(&path) {
            return Ok(None);
        }
        let file_id = FileId(0);

        if let Some(document) = self.get_cloned(&file_id) {
            return Ok(Some(document));
        }

        // self.add_file_to_projects(&file_id)?;

        let data = fs::read(&path)?;
        let text = String::from_utf8_lossy(&data).into_owned();
        let document = self.handle_open_document(&Arc::new(uri), text)?;
        self.resolve_missing_includes();

        Ok(Some(document))
    }

    pub fn reload(&mut self, path: PathBuf) -> anyhow::Result<Option<Document>> {
        let mut uri = Url::from_file_path(&path).map_err(|err| {
            anyhow!(
                "Failed to convert path to URI while loading a file: {:?}",
                err
            )
        })?;
        normalize_uri(&mut uri);
        if !self.is_sourcepawn_file(&path) {
            return Ok(None);
        }
        let _file_id = FileId(0);

        let data = fs::read(&path)?;
        let text = String::from_utf8_lossy(&data).into_owned();
        let document = self.handle_open_document(&Arc::new(uri), text)?;
        self.resolve_missing_includes();

        // self.remove_file_from_projects(&file_id);
        // self.add_file_to_projects(&file_id)?;

        Ok(Some(document))
    }

    pub fn resolve_missing_includes(&mut self) {
        let mut to_reload = FxHashSet::default();
        for document in self.documents.values() {
            for missing_include in document.missing_includes.keys() {
                for document_ in self.documents.values() {
                    if document_.uri.as_str().contains(missing_include) {
                        to_reload.insert(document.file_id);
                    }
                }
            }
        }
        for file_id in to_reload {
            if let Some(document) = self.documents.get(&file_id) {
                let _ = self.handle_open_document(&document.uri.clone(), document.text.clone());
            }
        }
    }

    pub fn discover_documents(&mut self, base_path: &PathBuf) {
        log::debug!("Finding documents in {:?}", base_path);
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
                let file_id = FileId(0);

                if self.documents.contains_key(&file_id) {
                    continue;
                }
                let Ok(text) = read_to_string_lossy(entry.path().to_path_buf()) else {
                    log::error!("Failed to read file {:?} ", entry.path());
                    continue;
                };
                let document = Document::new(Arc::new(uri.clone()), file_id, text.clone());
                self.documents.insert(file_id, document);
            }
        }
    }

    pub fn handle_open_document(
        &mut self,
        uri: &Arc<Url>,
        text: String,
    ) -> Result<Document, io::Error> {
        log::trace!("Opening file {:?}", uri);
        let file_id = FileId(0);

        self.diagnostics.reset(uri);
        let prev_declarations = match self.documents.get(&file_id) {
            Some(document) => document.declarations.clone(),
            None => FxHashMap::default(),
        };
        let mut document = Document::new(uri.clone(), file_id, text);
        // self.preprocess_document(&mut document);
        self.add_sourcemod_include(&mut document);
        self.parse(&mut document).expect("Couldn't parse document");
        if !self.first_parse {
            // Don't try to find references yet, all the tokens might not be referenced.
            self.resolve_file_references(&file_id);
            self.sync_references(&mut document, prev_declarations);
        }
        log::trace!("Done opening file {:?}", uri);

        Ok(document)
    }

    fn add_sourcemod_include(&mut self, document: &mut Document) {
        let mut sourcemod_path = "sourcemod".to_string();
        if let Some(include_id) = self.resolve_import(&mut sourcemod_path, &document.uri, false) {
            self.add_include(document, include_id, sourcemod_path, Range::default());
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
                    to_reload.push(sub_doc.file_id);
                    break;
                }
            }
        }
        for file_id in to_reload {
            // resolve includes
            if let Some(doc_to_reload) = self.documents.get_mut(&file_id) {
                for (mut missing_inc_path, range) in doc_to_reload.missing_includes.clone() {
                    // TODO: The false in this method call may be problematic.
                    if let Some(include_uri) =
                        self.resolve_import(&mut missing_inc_path, &document.uri, false)
                    {
                        self.add_include(document, include_uri, missing_inc_path, range);
                    }
                }
            }
            self.resolve_file_references(&file_id);
        }
        for item in deleted_declarations.values() {
            let item = item.read();
            let references = item.references();
            if let Some(references) = references {
                for ref_ in references.iter() {
                    if let Some(ref_document) = self.documents.get_mut(&ref_.file_id) {
                        ref_document.unresolved_tokens.insert(item.name());
                    }
                }
            }
        }
        log::trace!("Done syncing references for document {:?}", document.uri);
    }

    // pub(crate) fn preprocess_document(
    //     &mut self,
    //     document: &mut Document,
    // ) -> Option<FxHashMap<String, Macro>> {
    //     log::trace!("Preprocessing document {:?}", document.uri);
    //     if !document.preprocessed_text.is_empty() || document.being_preprocessed {
    //         log::trace!("Skipped preprocessing document {:?}", document.uri);
    //         return Some(document.macros.clone());
    //     }
    //     document.being_preprocessed = true;
    //     let mut preprocessor = SourcepawnPreprocessor::new(document.uri.clone(), &document.text);
    //     let preprocessed_text = preprocessor
    //         .preprocess_input(
    //             &mut (|macros: &mut FxHashMap<String, Macro>,
    //                    path: String,
    //                    document_uri: &Url,
    //                    quoted: bool| {
    //                 self.extend_macros(macros, path, document_uri, quoted)
    //             }),
    //         )
    //         .unwrap_or_else(|err| {
    //             log::error!("{:?}", err);
    //             document.text.clone()
    //         });

    //     document.preprocessed_text = preprocessed_text;
    //     document.macros = preprocessor.macros.clone();
    //     document.offsets = preprocessor.offsets.clone();
    //     preprocessor
    //         .add_diagnostics(&mut self.diagnostics.get_mut(&document.uri).local_diagnostics);
    //     document
    //         .macro_symbols
    //         .extend(preprocessor.evaluated_define_symbols.iter().map(|token| {
    //             Arc::new(Token {
    //                 text: token.text(),
    //                 range: token.range,
    //             })
    //         }));
    //     document.being_preprocessed = false;
    //     log::trace!("Done preprocessing document {:?}", document.uri);

    //     Some(preprocessor.macros)
    // }

    // pub(crate) fn preprocess_document_by_id(
    //     &mut self,
    //     file_id: &FileId,
    // ) -> Option<FxHashMap<String, Macro>> {
    //     // let document_uri = Arc::new(self.vfs.lookup(*file_id).clone());
    //     let document_uri = Arc::new(Url::from_str("http://example.com").unwrap());
    //     log::trace!("Preprocessing document by uri {:?}", document_uri);
    //     if let Some(document) = self.documents.get(file_id) {
    //         // Don't reprocess the text if it has not changed.
    //         if !document.preprocessed_text.is_empty() || document.being_preprocessed {
    //             log::trace!("Skipped preprocessing document by uri {:?}", document_uri);
    //             return Some(document.macros.clone());
    //         }
    //     }
    //     if let Some(document) = self.documents.get_mut(file_id) {
    //         document.being_preprocessed = true;
    //     }
    //     if let Some(text) = self.get_text(file_id) {
    //         let mut preprocessor = SourcepawnPreprocessor::new(document_uri, &text);
    //         let preprocessed_text = preprocessor
    //             .preprocess_input(
    //                 &mut (|macros: &mut FxHashMap<String, Macro>,
    //                        path: String,
    //                        document_uri: &Url,
    //                        quoted: bool| {
    //                     self.extend_macros(macros, path, document_uri, quoted)
    //                 }),
    //             )
    //             .unwrap_or_else(|err| {
    //                 log::error!("{:?}", err);
    //                 text.clone()
    //             });

    //         if let Some(document) = self.documents.get_mut(file_id) {
    //             document.preprocessed_text = preprocessed_text;
    //             document.macros = preprocessor.macros.clone();
    //             preprocessor.add_diagnostics(
    //                 &mut self.diagnostics.get_mut(&document.uri).local_diagnostics,
    //             );
    //             document
    //                 .macro_symbols
    //                 .extend(preprocessor.evaluated_define_symbols.iter().map(|token| {
    //                     Arc::new(Token {
    //                         text: token.text(),
    //                         range: token.range,
    //                     })
    //                 }));
    //         }
    //         return Some(preprocessor.macros);
    //     }
    //     if let Some(document) = self.documents.get_mut(file_id) {
    //         document.being_preprocessed = false;
    //     }
    //     log::trace!("Done preprocessing document by uri {:?}", document_uri);

    //     None
    // }

    // pub(crate) fn extend_macros(
    //     &mut self,
    //     macros: &mut FxHashMap<String, Macro>,
    //     mut include_text: String,
    //     document_uri: &Url,
    //     quoted: bool,
    // ) -> anyhow::Result<()> {
    //     if let Some(file_id) =
    //         self.resolve_import(&mut include_text, &Arc::new(document_uri.clone()), quoted)
    //     {
    //         if let Some(include_macros) = self.preprocess_document_by_id(&file_id) {
    //             macros.extend(include_macros);
    //         }
    //         return Ok(());
    //     }

    //     Err(anyhow!(
    //         "Could not resolve include \"{}\" from path.",
    //         include_text
    //     ))
    // }

    pub fn parse(&mut self, document: &mut Document) -> anyhow::Result<()> {
        log::trace!("Parsing document {:?}", document.uri);
        let mut parser = spawn_parser();
        let tree = parser
            .parse(&document.preprocessed_text, None)
            .ok_or(anyhow!("Failed to parse document {:?}", document.uri))?;
        let root_node = tree.root_node();
        let mut walker = Parser {
            comments: vec![],
            deprecated: vec![],
            anon_enum_counter: 0,
            sp_items: &mut document.sp_items,
            declarations: &mut document.declarations,
            offsets: &document.offsets,
            source: &document.preprocessed_text,
            uri: document.uri.clone(),
            file_id: document.file_id,
        };

        let mut cursor = root_node.walk();

        let mut includes_to_add = Vec::new();

        for mut node in root_node.children(&mut cursor) {
            let kind = node.kind();
            let _ = match kind {
                "function_declaration" | "function_definition" => {
                    walker.parse_function(&node, None)
                }
                "global_variable_declaration" | "old_global_variable_declaration" => {
                    walker.parse_variable(&mut node, None)
                }
                "preproc_include" | "preproc_tryinclude" => match walker.parse_include(&mut node) {
                    Ok(include) => {
                        includes_to_add.push(include);
                        Ok(())
                    }
                    Err(err) => Err(err),
                },
                "enum" => walker.parse_enum(&mut node),
                "preproc_define" => walker.parse_define(&mut node),
                "methodmap" => walker.parse_methodmap(&mut node),
                "typedef" => walker.parse_typedef(&node),
                "typeset" => walker.parse_typeset(&node),
                "preproc_macro" => walker.parse_macro(&mut node),
                "enum_struct" => walker.parse_enum_struct(&mut node),
                "comment" => {
                    walker.push_comment(node);
                    let Some(item) = walker.sp_items.pop() else {
                        continue;
                    };
                    walker.push_inline_comment(&item);
                    walker.sp_items.push(item);
                    Ok(())
                }
                "preproc_pragma" => walker.push_deprecated(node),
                _ => continue,
            };
        }
        self.add_includes(includes_to_add, document);
        document.parsed = true;
        document.extract_tokens(root_node);
        document.add_macro_symbols();
        self.diagnostics.get_syntax_error_diagnostics(
            &document.uri,
            &document.preprocessed_text,
            root_node,
            self.environment.options.disable_syntax_linter,
        );
        self.documents.insert(document.file_id, document.clone());
        self.read_unscanned_imports(&document.includes);
        log::trace!("Done parsing document {:?}", document.uri);

        Ok(())
    }

    fn add_includes(
        &mut self,
        includes_to_add: Vec<parser::include_parser::Include>,
        document: &mut Document,
    ) {
        for mut include in includes_to_add {
            match self.resolve_import(&mut include.path, &document.uri, include.quoted) {
                Some(uri) => {
                    self.add_include(document, uri, include.path, include.range);
                }
                None => {
                    document
                        .missing_includes
                        .insert(include.path, include.range);
                }
            };
        }
    }

    pub(crate) fn read_unscanned_imports(&mut self, includes: &FxHashMap<FileId, Token>) {
        for include_uri in includes.keys() {
            let document = self
                .get_cloned(include_uri)
                .expect("Include does not exist.");
            if document.parsed {
                continue;
            }
            let document = self
                .handle_open_document(&document.uri, document.text)
                .expect("Couldn't parse file");
            self.read_unscanned_imports(&document.includes)
        }
    }

    /// Resolve all the references in a project, given the [file_id](FileId) of a file in the project.
    /// Will not run if the main file has already been resolved at least once.
    /// Returns [None] if it did not run and [Some(file_id)] if it did, with [file_id](FileId) being the id of the
    /// main file.
    ///
    /// # Arguments
    /// * `uri` - The [uri](Url) of a file in the project. Does not have to be the root.
    pub fn resolve_project_references(&mut self, _uri: &Url) -> Option<FileId> {
        log::trace!("Resolving project references.");
        let _file_id = FileId(0);
        let main_id = FileId(0);
        // let main_id = self.projects.find_root_from_id(file_id)?.file_id;
        let file_ids: Vec<FileId> = {
            let mut includes = FxHashSet::default();
            includes.insert(main_id);
            if let Some(document) = self.documents.get(&main_id) {
                if document.is_resolved() {
                    // Main file has already been resolved, assume the rest of the project has been too.
                    return None;
                }
                self.get_included_files(document, &mut includes);
                includes.iter().cloned().collect()
            } else {
                self.documents.values().map(|doc| doc.file_id).collect()
            }
        };
        file_ids.iter().for_each(|file_id: &FileId| {
            self.resolve_file_references(file_id);
        });
        log::trace!("Done resolving project references.");

        Some(main_id)
    }

    pub fn get_all_files_in_folder(&self, folder_uri: &Url) -> Vec<Url> {
        let mut children = vec![];
        for document in self.documents.values() {
            if document.uri.as_str().contains(folder_uri.as_str()) {
                children.push(document.uri.as_ref().clone());
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
