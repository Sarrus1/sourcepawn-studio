use std::sync::Arc;

use lazy_static::lazy_static;
use lsp_types::{Position, Range};
use regex::Regex;
use sourcepawn_lexer::{PreprocDir, Symbol, TokenKind};
use vfs::{AnchoredPath, FileId};

use crate::{FileExtension, SourceDatabase};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IncludeType {
    /// #include <foo>
    Include,

    /// #tryinclude <foo>
    TryInclude,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IncludeKind {
    /// #include <foo>
    Chevrons,

    /// #include "foo"
    Quotes,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Include {
    id: FileId,
    kind: IncludeKind,
    type_: IncludeType,
    extension: FileExtension,
}

impl Include {
    pub fn new(
        id: FileId,
        kind: IncludeKind,
        type_: IncludeType,
        extension: FileExtension,
    ) -> Self {
        Self {
            id,
            kind,
            type_,
            extension,
        }
    }

    pub fn file_id(&self) -> FileId {
        self.id
    }

    pub fn kind(&self) -> IncludeKind {
        self.kind
    }

    pub fn type_(&self) -> IncludeType {
        self.type_
    }

    pub fn extension(&self) -> FileExtension {
        self.extension
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UnresolvedInclude {
    pub file_id: FileId,
    pub range: lsp_types::Range,
    pub path: String,
}

lazy_static! {
    pub static ref RE_CHEVRON: Regex = Regex::new(r"<([^>]+)>").unwrap();
    pub static ref RE_QUOTE: Regex = Regex::new("\"([^>]+)\"").unwrap();
}

/// Returns all resolved and unresolved includes in the file.
///
/// # Note
/// Does not return includes that are in subincludes, i.e this function is not recursive.
pub(crate) fn file_includes_query(
    db: &dyn SourceDatabase,
    file_id: FileId,
) -> (Arc<Vec<Include>>, Arc<Vec<UnresolvedInclude>>) {
    let mut res = vec![];
    let mut unresolved = vec![];

    // Include sourcemod by default
    if let Some(include_file_id) = db.resolve_path_relative_to_roots("sourcemod.inc") {
        res.push(Include::new(
            include_file_id,
            IncludeKind::Chevrons,
            IncludeType::TryInclude,
            FileExtension::Inc,
        ));
    }

    let input = db.file_text(file_id);
    let lexer = sourcepawn_lexer::SourcepawnLexer::new(&input);
    for symbol in lexer {
        match symbol.token_kind {
            TokenKind::PreprocDir(PreprocDir::MInclude)
            | TokenKind::PreprocDir(PreprocDir::MTryinclude) => {
                let type_ = if symbol.token_kind == TokenKind::PreprocDir(PreprocDir::MInclude) {
                    IncludeType::Include
                } else {
                    IncludeType::TryInclude
                };
                let text = symbol.inline_text().trim().to_string();
                let symbol = Symbol::new(
                    symbol.token_kind,
                    Some(&text),
                    Range::new(
                        Position::new(symbol.range.start.line, symbol.range.start.character),
                        Position::new(symbol.range.start.line, text.len() as u32),
                    ),
                    symbol.delta,
                );

                let mut kind = IncludeKind::Chevrons;
                let mut path = None;
                let mut ext = None;

                if let Some(m) = RE_QUOTE.captures(&text).and_then(|caps| caps.get(1)) {
                    kind = IncludeKind::Quotes;
                    let mut raw_path = m.as_str().to_string();
                    let raw_ext = infer_include_ext(&mut raw_path);
                    if let Some(include_file_id) =
                        db.resolve_path(AnchoredPath::new(file_id, &raw_path))
                    {
                        res.push(Include::new(include_file_id, kind, type_, raw_ext));
                        continue;
                    }
                    // Hack to detect `include` folders when it's a relative include.
                    let raw_path_with_include = format!("include/{}", raw_path);
                    if let Some(include_file_id) =
                        db.resolve_path(AnchoredPath::new(file_id, &raw_path_with_include))
                    {
                        res.push(Include::new(include_file_id, kind, type_, raw_ext));
                        continue;
                    }
                    path = Some(raw_path);
                    ext = Some(raw_ext);
                }

                if path.is_none() {
                    if let Some(m) = RE_CHEVRON.captures(&text).and_then(|caps| caps.get(1)) {
                        kind = IncludeKind::Chevrons;
                        let mut raw_path = m.as_str().to_string();
                        ext = Some(infer_include_ext(&mut raw_path));
                        path = Some(raw_path);
                    }
                }
                let (path, ext) = match (path, ext) {
                    (Some(path), Some(ext)) => (path, ext),
                    (Some(path), _) => {
                        if type_ == IncludeType::Include {
                            // TODO: Optional diagnostic for tryinclude ?
                            // FIXME: Emit the diagnostics in the preprocessor. This would make more sense as some
                            // includes might be disabled by the preprocessor.
                            unresolved.push(UnresolvedInclude {
                                file_id,
                                range: symbol.range,
                                path,
                            })
                        }
                        continue;
                    }
                    _ => continue,
                };
                match db.resolve_path_relative_to_roots(&path) {
                    Some(include_file_id) => {
                        res.push(Include::new(include_file_id, kind, type_, ext));
                        continue;
                    }
                    None => {
                        if type_ == IncludeType::Include {
                            // TODO: Optional diagnostic for tryinclude ?
                            // FIXME: Emit the diagnostics in the preprocessor. This would make more sense as some
                            // includes might be disabled by the preprocessor.
                            unresolved.push(UnresolvedInclude {
                                file_id,
                                range: symbol.range,
                                path,
                            })
                        }
                    }
                }
            }
            _ => (),
        };
    }

    (Arc::new(res), Arc::new(unresolved))
}

/// Mutate the include path to add `.inc` if necessary and return the detected file extension.
pub fn infer_include_ext(path: &mut String) -> FileExtension {
    if path.ends_with(".sp") {
        FileExtension::Sp
    } else if path.ends_with(".inc") {
        FileExtension::Inc
    } else {
        *path += ".inc";
        FileExtension::Inc
    }
}
