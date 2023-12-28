use std::sync::Arc;

use graph::Graph;
use vfs::FileId;

mod change;
mod graph;

pub use change::Change;

pub trait FileLoader {
    /// Text of the file.
    fn file_text(&self, file_id: FileId) -> Arc<str>;
}

#[derive(Debug, Clone)]
pub struct Tree(tree_sitter::Tree);

/// Helper function to get the name of the field a node is in.
///
/// # Arguments
///
/// * `node` - The node to get the field name of.
///
/// # Notes
///
/// This has O(n) time complexity try not to use it for large nodes.
pub fn field_name(node: &tree_sitter::Node) -> Option<&'static str> {
    let parent = node.parent()?;
    let mut cursor = parent.walk();
    parent.children(&mut cursor).find(|child| child == node)?;
    cursor.field_name()
}

impl Tree {
    pub fn tree(&self) -> &tree_sitter::Tree {
        &self.0
    }

    pub fn root_node(&self) -> tree_sitter::Node {
        self.tree().root_node()
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
    // Parses the file into the syntax tree.
    #[salsa::invoke(parse_query)]
    fn parse(&self, file_id: FileId) -> Tree;

    #[salsa::input]
    fn projects_graph(&self) -> Graph;

    #[salsa::invoke(Graph::projet_root_query)]
    fn projet_root(&self, file_id: FileId) -> Option<FileId>;
}

fn parse_query(db: &dyn SourceDatabase, file_id: FileId) -> Tree {
    tracing::info!("Parsing {}", file_id);
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(tree_sitter_sourcepawn::language())
        .expect("Failed to set language");
    let text = db.file_text(file_id);
    parser
        .parse(text.as_ref(), None)
        .expect("Failed to parse a file.")
        .into()
}

/// We don't want to give HIR knowledge of source roots, hence we extract these
/// methods into a separate DB.
#[salsa::query_group(SourceDatabaseExtStorage)]
pub trait SourceDatabaseExt: SourceDatabase {
    #[salsa::input]
    fn file_text(&self, file_id: FileId) -> Arc<str>;
}

/// Silly workaround for cyclic deps between the traits
pub struct FileLoaderDelegate<T>(pub T);

impl<T: SourceDatabaseExt> FileLoader for FileLoaderDelegate<&'_ T> {
    fn file_text(&self, file_id: FileId) -> Arc<str> {
        SourceDatabaseExt::file_text(self.0, file_id)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FilePosition {
    pub file_id: FileId,
    pub position: lsp_types::Position,
}

pub trait Upcast<T: ?Sized> {
    fn upcast(&self) -> &T;
}
