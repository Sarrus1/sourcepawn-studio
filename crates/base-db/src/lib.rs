use std::sync::Arc;
use std::{
    hash::{Hash, Hasher},
    ops::Index,
};

use graph::Graph;
use la_arena::{Arena, Idx};
use lsp_types::Range;
use syntax::utils::ts_range_to_lsp_range;
use vfs::FileId;

mod change;
mod graph;

pub use change::Change;

pub trait FileLoader {
    /// Text of the file.
    fn file_text(&self, file_id: FileId) -> Arc<str>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    data: Arena<Node>,
}

impl Tree {
    pub fn data(&self) -> &Arena<Node> {
        &self.data
    }
}

// Node struct to represent each node in the tree.
#[derive(Debug, Clone)]
pub struct Node {
    node_type: String,
    field_name: Option<String>,
    start_byte: usize,
    end_byte: usize,
    range: Range,
    parent: Option<Idx<Node>>, // Using indices to refer to other nodes
    children: Vec<Idx<Node>>,
}

// FIXME: Was it worth it to wrap around tree-sitter's Node/Tree?

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.node_type == other.node_type
            && self.start_byte == other.start_byte
            && self.parent == other.parent
            && self.children == other.children
    }
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node_type.hash(state);
        self.field_name.hash(state);
        self.start_byte.hash(state);
        self.end_byte.hash(state);
        self.parent.hash(state);
        self.children.hash(state);
    }
}

impl Node {
    pub fn children(&self) -> &[Idx<Node>] {
        &self.children
    }

    pub fn kind(&self) -> &str {
        &self.node_type
    }

    pub fn child_by_field_name<'a>(&'a self, name: &str, tree: &'a Tree) -> Option<&Node> {
        self.children()
            .iter()
            .find(|&child| tree[*child].field_name == Some(name.to_string()))
            .map(|idx| &tree[*idx])
    }

    pub fn utf8_text<'a>(&self, text: &'a [u8]) -> Option<&'a str> {
        std::str::from_utf8(&text[self.start_byte..self.end_byte]).ok()
    }

    pub fn start_byte(&self) -> usize {
        self.start_byte
    }

    pub fn end_byte(&self) -> usize {
        self.end_byte
    }

    pub fn range(&self) -> Range {
        self.range
    }
}

impl Eq for Node {}

impl From<tree_sitter::Tree> for Tree {
    fn from(ts_tree: tree_sitter::Tree) -> Self {
        let mut arena: Arena<Node> = Arena::new();
        Tree::convert_node(&mut arena, None, ts_tree.root_node(), None);

        Tree { data: arena }
    }
}

impl Tree {
    fn convert_node(
        arena: &mut Arena<Node>,
        parent_idx: Option<Idx<Node>>,
        ts_node: tree_sitter::Node,
        field_name: Option<&str>,
    ) -> Option<Idx<Node>> {
        // Create a Node from the tree-sitter node
        let node_idx = arena.alloc(Node {
            node_type: ts_node.kind().to_string(),
            field_name: field_name.map(|s| s.to_string()),
            start_byte: ts_node.start_byte(),
            end_byte: ts_node.end_byte(),
            range: ts_range_to_lsp_range(&ts_node.range()),
            parent: parent_idx,
            children: Vec::new(),
        });

        // Traverse children of the tree-sitter node and add them to the arena
        let mut cursor = ts_node.walk();
        if cursor.goto_first_child() {
            loop {
                if let Some(child_idx) =
                    Tree::convert_node(arena, Some(node_idx), cursor.node(), cursor.field_name())
                {
                    // Link the child to its parent
                    arena[node_idx].children.push(child_idx);
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        Some(node_idx)
    }

    pub fn root_node(&self) -> &Node {
        self.data.iter().next().unwrap().1
    }

    // Find the innermost node containing a given position.
    pub fn node_from_pos(&self, pos: lsp_types::Position) -> Option<Idx<Node>> {
        self.find_innermost_node_recursive(self.data.iter().next()?.0, pos, None)
    }

    fn find_innermost_node_recursive(
        &self,
        current_idx: Idx<Node>,
        pos: lsp_types::Position,
        best_match: Option<(Idx<Node>, usize)>,
    ) -> Option<Idx<Node>> {
        let current_node = &self[current_idx];

        // Check if the current node contains the byte position
        if pos >= current_node.range.start && pos < current_node.range.end {
            let range = current_node.end_byte - current_node.start_byte;
            let best_match = match best_match {
                Some((_, best_range)) if range < best_range => Some((current_idx, range)),
                None => Some((current_idx, range)),
                _ => best_match,
            };

            // Recursively search in the children for a better match
            for &child_idx in &current_node.children {
                let best_match = self.find_innermost_node_recursive(child_idx, pos, best_match);
                if best_match.is_some() {
                    return best_match;
                }
            }

            return best_match.map(|(idx, _)| idx);
        }

        None
    }
}

impl Index<Idx<Node>> for Tree {
    type Output = Node;
    fn index(&self, index: Idx<Node>) -> &Self::Output {
        &self.data()[index]
    }
}

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
