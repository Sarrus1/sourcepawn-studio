use anyhow::anyhow;
use fxhash::{FxHashMap, FxHashSet};
use lazy_static::lazy_static;
use lsp_types::Range;
use lsp_types::Url;
use parking_lot::RwLock;
use parser::build_v_range;
use preprocessor::{Macro, Offset};
use semantic_analyzer::{SPToken, Token};
use std::{path::PathBuf, sync::Arc};
use strip_bom::StripBom;
use syntax::SPItem;
use tree_sitter::{Node, Query, QueryCursor};
use vfs::FileId;

lazy_static! {
    static ref METHOD_QUERY: Query = Query::new(
        tree_sitter_sourcepawn::language(),
        "[(field_access) @method] (scope_access) @method (array_scope_access) @method",
    )
    .expect("Could not build methods query.");
    static ref SYMBOL_QUERY: Query = {
        Query::new(
            tree_sitter_sourcepawn::language(),
            "[(symbol) @symbol (this) @symbol]",
        )
        .expect("Could not build symbols query.")
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FileExtension {
    #[default]
    Sp,
    Inc,
}

pub fn uri_to_file_extension(uri: &Url) -> Option<FileExtension> {
    let path = uri.to_file_path().ok()?;
    let extension = path.extension()?;
    match extension.to_str()? {
        "sp" => Some(FileExtension::Sp),
        "inc" => Some(FileExtension::Inc),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub uri: Arc<Url>,
    pub file_id: FileId,
    extension: FileExtension,
    pub text: String,
    pub preprocessed_text: String,
    pub(super) being_preprocessed: bool,
    pub sp_items: Vec<Arc<RwLock<SPItem>>>,
    pub(crate) includes: FxHashMap<FileId, Token>,
    pub parsed: bool,
    resolved: bool,
    pub(crate) tokens: Vec<SPToken>,
    pub missing_includes: FxHashMap<String, Range>,
    pub unresolved_tokens: FxHashSet<String>,
    pub declarations: FxHashMap<String, Arc<RwLock<SPItem>>>,
    pub(crate) macros: FxHashMap<String, Macro>,
    pub(crate) macro_symbols: Vec<Arc<Token>>,
    pub(crate) offsets: FxHashMap<u32, Vec<Offset>>,
}

impl Document {
    pub fn new(uri: Arc<Url>, file_id: FileId, text: String) -> Self {
        Self {
            extension: {
                if let Ok(file_path) = uri.to_file_path() {
                    if file_path.ends_with(".sp") {
                        FileExtension::Sp
                    } else {
                        FileExtension::Inc
                    }
                } else {
                    // This happens when using the debug preprocessed_text command in VSCode.
                    FileExtension::Sp
                }
            },
            file_id,
            uri,
            preprocessed_text: String::new(),
            being_preprocessed: false,
            text: text.strip_bom().to_string(),
            sp_items: vec![],
            includes: FxHashMap::default(),
            parsed: false,
            tokens: vec![],
            missing_includes: FxHashMap::default(),
            unresolved_tokens: FxHashSet::default(),
            declarations: FxHashMap::default(),
            macros: FxHashMap::default(),
            macro_symbols: vec![],
            offsets: FxHashMap::default(),
            resolved: false,
        }
    }

    /// Return `true` if the document tokens have been resolved at least once, `false` otherwise.
    pub fn is_resolved(&self) -> bool {
        self.resolved
    }

    /// Mark the document tokens as resolved at least once.
    pub fn mark_as_resolved(&mut self) {
        self.resolved = true;
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn extension(&self) -> FileExtension {
        self.extension
    }

    pub(crate) fn path(&self) -> anyhow::Result<PathBuf> {
        let path = self
            .uri
            .to_file_path()
            .map_err(|e| anyhow!("Failed to convert URI to file path: {:?}.", e))?;

        Ok(path)
    }

    pub fn uri(&self) -> Url {
        (*self.uri.as_ref()).clone()
    }

    pub(crate) fn extract_tokens(&mut self, root_node: Node) {
        let mut symbols: FxHashMap<tree_sitter::Range, Arc<Token>> = FxHashMap::default();
        let mut cursor = QueryCursor::new();
        let matches = cursor.captures(&SYMBOL_QUERY, root_node, self.preprocessed_text.as_bytes());
        for (match_, _) in matches {
            for capture in match_.captures.iter() {
                symbols.insert(
                    capture.node.range(),
                    Arc::new(Token::new(capture.node, &self.preprocessed_text)),
                );
            }
        }
        let mut method_symbols: FxHashMap<tree_sitter::Range, (Arc<Token>, Arc<Token>)> =
            FxHashMap::default();
        let matches = cursor.captures(&METHOD_QUERY, root_node, self.preprocessed_text.as_bytes());
        for (match_, _) in matches {
            for capture in match_.captures.iter() {
                let mut sub_cursor = QueryCursor::new();
                let sub_matches = sub_cursor.captures(
                    &SYMBOL_QUERY,
                    capture.node,
                    self.preprocessed_text.as_bytes(),
                );
                for (i, (sub_match, _)) in sub_matches.enumerate() {
                    if i > 0 || sub_match.captures.is_empty() {
                        // Assume the first symbol is the item we are accessing the field of.
                        break;
                    }
                    if let Some(field) = capture.node.child_by_field_name("field") {
                        method_symbols.insert(
                            field.range(),
                            (
                                Arc::new(Token::new(
                                    sub_match.captures[0].node,
                                    &self.preprocessed_text,
                                )),
                                Arc::new(Token::new(field, &self.preprocessed_text)),
                            ),
                        );
                    }
                }
            }
        }

        // Remove methods that are also symbols to avoid overlapping tokens in resolution.
        for range in method_symbols.keys() {
            symbols.remove(range);
        }

        self.tokens
            .extend(symbols.values().map(|t| SPToken::Symbol(t.clone())));
        self.tokens.extend(
            method_symbols
                .values()
                .map(|(t1, t2)| SPToken::Method((t1.clone(), t2.clone()))),
        );
    }

    pub fn add_macro_symbols(&mut self) {
        self.tokens.extend(
            self.macro_symbols
                .iter()
                .map(|t| SPToken::Symbol(t.clone())),
        );
    }

    pub fn line(&self, line_nb: u32) -> Option<&str> {
        let mut len = 0;
        for (i, line) in self.preprocessed_text.lines().enumerate() {
            len += 1;
            if i == line_nb as usize {
                return Some(line);
            }
        }
        if len == line_nb && self.preprocessed_text.ends_with('\n') {
            return Some("");
        }

        None
    }

    pub fn get_sp_items(&self) -> Vec<Arc<RwLock<SPItem>>> {
        let mut sp_items = vec![];
        for item in self.sp_items.iter() {
            sp_items.push(item.clone());
        }

        sp_items
    }

    pub fn get_sp_items_flat(&self) -> Vec<Arc<RwLock<SPItem>>> {
        let mut sp_items = vec![];
        for item in self.sp_items.iter() {
            sp_items.push(item.clone());
            match &*item.read() {
                SPItem::Function(function_item) => {
                    for child_item in function_item.children.iter() {
                        sp_items.push(child_item.clone())
                    }
                }
                SPItem::Enum(enum_item) => {
                    for child_item in enum_item.children.iter() {
                        sp_items.push(child_item.clone())
                    }
                }
                SPItem::EnumStruct(es_item) => {
                    for child_item in es_item.children.iter() {
                        sp_items.push(child_item.clone());
                        match &*child_item.read() {
                            SPItem::Function(method_item) => {
                                for sub_child_item in method_item.children.iter() {
                                    sp_items.push(sub_child_item.clone());
                                }
                            }
                            SPItem::EnumMember(_)
                            | SPItem::Typedef(_)
                            | SPItem::Typeset(_)
                            | SPItem::Variable(_)
                            | SPItem::Property(_)
                            | SPItem::Include(_)
                            | SPItem::Methodmap(_)
                            | SPItem::Enum(_)
                            | SPItem::EnumStruct(_)
                            | SPItem::Define(_) => {}
                        }
                    }
                }
                SPItem::Methodmap(mm_item) => {
                    for child_item in mm_item.children.iter() {
                        sp_items.push(child_item.clone());
                        match &*child_item.read() {
                            SPItem::Function(method_item) => {
                                for sub_child_item in method_item.children.iter() {
                                    sp_items.push(sub_child_item.clone());
                                }
                            }
                            SPItem::EnumMember(_)
                            | SPItem::Typedef(_)
                            | SPItem::Typeset(_)
                            | SPItem::Property(_)
                            | SPItem::Variable(_)
                            | SPItem::Include(_)
                            | SPItem::Methodmap(_)
                            | SPItem::Enum(_)
                            | SPItem::EnumStruct(_)
                            | SPItem::Define(_) => {}
                        }
                    }
                }
                SPItem::Typeset(ts_item) => {
                    for child_item in ts_item.children.iter() {
                        sp_items.push(child_item.clone())
                    }
                }
                SPItem::Variable(_)
                | SPItem::Typedef(_)
                | SPItem::EnumMember(_)
                | SPItem::Property(_)
                | SPItem::Include(_)
                | SPItem::Define(_) => {}
            }
        }

        sp_items
    }

    pub fn build_v_range(&self, range: &Range) -> Range {
        build_v_range(&self.offsets, range)
    }
}
