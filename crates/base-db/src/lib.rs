use std::{hash::Hash, sync::Arc};

use include::file_includes_query;
use input::{SourceRoot, SourceRootId};
use syntax::utils::lsp_position_to_ts_point;
use text_size::{TextRange, TextSize};
use vfs::{AnchoredPath, FileId};

mod change;
mod graph;
mod include;
mod input;

pub use {
    change::Change,
    graph::{Graph, SubGraph},
    include::{
        infer_include_ext, Include, IncludeKind, IncludeType, UnresolvedInclude, RE_CHEVRON,
        RE_QUOTE,
    },
    input::SourceRootConfig,
};

pub const DEFAULT_PARSE_LRU_CAP: usize = 128;

pub trait FileLoader {
    /// Text of the file.
    fn file_text(&self, file_id: FileId) -> Arc<str>;

    /// Known files.
    fn known_files(&self) -> Vec<(FileId, FileExtension)>;

    /// Resolve a path to a file.
    fn resolve_path(&self, path: AnchoredPath<'_>) -> Option<FileId>;

    /// Resolve a path relative to the roots.
    fn resolve_path_relative_to_roots(&self, path: &str) -> Option<FileId>;
}

#[derive(Debug, Clone)]
pub struct Tree(tree_sitter::Tree);

/// Helper function to check if a node is a name node.
///
/// # Arguments
///
/// * `node` - The node to check for.
pub fn is_name_node(node: &tree_sitter::Node) -> bool {
    node.parent()
        .and_then(|parent| parent.child_by_field_name("name"))
        .map(|child| child == *node)
        .unwrap_or(false)
}

/// Helper function to check if a node is a field receiver node.
///
/// # Arguments
///
/// * `node` - The node to check for.
pub fn is_field_receiver_node(node: &tree_sitter::Node) -> bool {
    node.parent()
        .and_then(|parent| parent.child_by_field_name("field"))
        .map(|child| child == *node)
        .unwrap_or(false)
}

impl Tree {
    pub fn tree(&self) -> &tree_sitter::Tree {
        &self.0
    }

    pub fn edit(&mut self, edit: &tree_sitter::InputEdit) {
        self.0.edit(edit);
    }

    pub fn root_node(&self) -> tree_sitter::Node {
        self.tree().root_node()
    }

    pub fn covering_element(&self, range: lsp_types::Range) -> Option<tree_sitter::Node> {
        let start = lsp_position_to_ts_point(&range.start);
        let end = lsp_position_to_ts_point(&range.end);
        self.root_node().descendant_for_point_range(start, end)
    }
}

impl From<tree_sitter::Tree> for Tree {
    fn from(ts_tree: tree_sitter::Tree) -> Self {
        Tree(ts_tree)
    }
}

impl From<Tree> for tree_sitter::Tree {
    fn from(tree: Tree) -> Self {
        tree.0
    }
}

impl From<&Tree> for tree_sitter::Tree {
    fn from(tree: &Tree) -> Self {
        tree.0.clone()
    }
}

impl PartialEq for Tree {
    fn eq(&self, other: &Self) -> bool {
        self.0.root_node() == other.0.root_node()
    }
}

impl Eq for Tree {}

/// Database which stores all significant input facts: source code and project
/// model. Everything else in rust-analyzer is derived from these queries.
#[salsa::query_group(SourceDatabaseStorage)]
pub trait SourceDatabase: FileLoader + std::fmt::Debug {
    #[salsa::invoke(file_includes_query)]
    fn file_includes(&self, file_id: FileId) -> (Arc<Vec<Include>>, Arc<Vec<UnresolvedInclude>>);

    #[salsa::invoke(graph::Graph::graph_query)]
    fn graph(&self) -> Arc<graph::Graph>;

    #[salsa::invoke(graph::Graph::projet_subgraph_query)]
    fn projet_subgraph(&self, file_id: FileId) -> Option<Arc<graph::SubGraph>>;
}

/// We don't want to give HIR knowledge of source roots, hence we extract these
/// methods into a separate DB.
#[salsa::query_group(SourceDatabaseExtStorage)]
pub trait SourceDatabaseExt: SourceDatabase {
    /// Contents of the file.
    #[salsa::input]
    fn file_text(&self, file_id: FileId) -> Arc<str>;

    #[salsa::input]
    fn known_files(&self) -> Vec<(FileId, FileExtension)>;

    /// Source root of the file.
    #[salsa::input]
    fn file_source_root(&self, file_id: FileId) -> SourceRootId;

    /// Contents of the source root.
    #[salsa::input]
    fn source_root(&self, id: SourceRootId) -> Arc<SourceRoot>;

    /// Source roots
    #[salsa::input]
    fn source_roots(&self) -> Vec<Arc<SourceRoot>>;
}

/// Silly workaround for cyclic deps between the traits
pub struct FileLoaderDelegate<T>(pub T);

impl<T: SourceDatabaseExt> FileLoader for FileLoaderDelegate<&'_ T> {
    fn file_text(&self, file_id: FileId) -> Arc<str> {
        SourceDatabaseExt::file_text(self.0, file_id)
    }
    fn known_files(&self) -> Vec<(FileId, FileExtension)> {
        SourceDatabaseExt::known_files(self.0)
    }
    fn resolve_path(&self, path: AnchoredPath<'_>) -> Option<FileId> {
        // FIXME: this *somehow* should be platform agnostic...
        let source_root = self.0.file_source_root(path.anchor);
        let source_root = self.0.source_root(source_root);
        source_root.resolve_path(&path)
    }
    fn resolve_path_relative_to_roots(&self, path: &str) -> Option<FileId> {
        for source_root in self.0.source_roots() {
            if let Some(file_id) = source_root.resolve_path_relative_to_root(path) {
                return Some(file_id);
            }
        }
        None
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FilePosition {
    pub file_id: FileId,
    pub offset: TextSize,
}

impl FilePosition {
    pub fn raw_offset(&self) -> u32 {
        self.offset.into()
    }

    pub fn raw_offset_usize(&self) -> usize {
        self.raw_offset() as usize
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileRange {
    pub file_id: FileId,
    pub range: TextRange,
}

impl Hash for FileRange {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.file_id.hash(state);
        self.range.start().hash(state);
        self.range.end().hash(state);
    }
}

pub trait Upcast<T: ?Sized> {
    fn upcast(&self) -> &T;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub enum FileExtension {
    #[default]
    Sp,
    Inc,
}

impl TryFrom<&str> for FileExtension {
    type Error = &'static str;

    fn try_from(extension: &str) -> Result<Self, Self::Error> {
        match extension {
            "sp" => Ok(FileExtension::Sp),
            "inc" => Ok(FileExtension::Inc),
            _ => Err(""),
        }
    }
}
