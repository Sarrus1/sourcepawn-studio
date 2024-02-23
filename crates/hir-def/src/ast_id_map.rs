use core::fmt;
use std::fmt::Display;
use std::ops::Index;
use std::sync::Arc;

use base_db::Tree;
use fxhash::FxHashMap;
use la_arena::{Arena, Idx};
use syntax::TSKind;
use vfs::FileId;

use crate::DefDatabase;

/// Not a _pointer_ in the memory sense, rather a location in a [Tree-Sitter](tree_sitter) syntax tree.
/// [NodePtr] is used by the [AstIdMap] to provide an incremental computation barrier for Salsa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodePtr {
    /// Kind of the [node](tree_sitter::Node).
    ///
    /// Sometimes two nodes are exactly overlapping (same start and end), for instance a single function
    /// in a source file will have the same start and end positions as the `source_file` node.
    /// `kind` allows to disambiguate between two overlapping nodes.
    kind: TSKind,

    /// Start byte of the [node](tree_sitter::Node).
    start_byte: usize,

    /// End byte of the [node](tree_sitter::Node).
    end_byte: usize,
}

impl From<&tree_sitter::Node<'_>> for NodePtr {
    fn from(node: &tree_sitter::Node<'_>) -> Self {
        NodePtr {
            kind: TSKind::from(*node),
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
        }
    }
}

impl NodePtr {
    pub fn to_node(self, tree: &'_ Tree) -> tree_sitter::Node<'_> {
        let mut node = tree.root_node();
        loop {
            if node.start_byte() == self.start_byte
                && node.end_byte() == self.end_byte
                && TSKind::from(node) == self.kind
            {
                return node;
            }
            let mut found = false;
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.start_byte() <= self.start_byte && child.end_byte() >= self.end_byte {
                    node = child;
                    found = true;
                    break;
                }
            }
            if !found {
                panic!("failed to find node")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AstId {
    raw: Idx<NodePtr>,
}

impl Display for AstId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AstId({})", self.raw.into_raw().into_u32())
    }
}

impl From<u32> for AstId {
    fn from(raw: u32) -> Self {
        AstId {
            raw: Idx::from_raw(raw.into()),
        }
    }
}

impl AstId {
    pub fn to_u32(self) -> u32 {
        self.raw.into_raw().into_u32()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstIdMap {
    arena: Arena<NodePtr>,
    map: FxHashMap<NodePtr, AstId>,
}

impl AstIdMap {
    pub fn from_tree(db: &dyn DefDatabase, file_id: FileId) -> Arc<Self> {
        let tree = db.parse(file_id);
        Arc::new(AstIdMap::from_source(&tree.root_node()))
    }

    fn from_source(root_node: &tree_sitter::Node) -> Self {
        assert!(root_node.parent().is_none());
        let mut arena = Arena::default();
        let mut map = FxHashMap::default();
        bdfs(root_node, &mut |node: tree_sitter::Node<'_>| {
            match TSKind::from(node) {
                TSKind::source_file => (),
                TSKind::global_variable_declaration | TSKind::variable_declaration_statement => {
                    for child in node.children(&mut node.walk()) {
                        if TSKind::from(child) == TSKind::variable_declaration {
                            let node_ptr = NodePtr::from(&child);
                            let ast_id = arena.alloc(node_ptr);
                            map.insert(node_ptr, AstId { raw: ast_id });
                        }
                    }
                }
                _ => {
                    let node_ptr = NodePtr::from(&node);
                    let ast_id = arena.alloc(node_ptr);
                    map.insert(node_ptr, AstId { raw: ast_id });
                }
            }
            matches!(
                TSKind::from(node),
                TSKind::function_definition
                    | TSKind::methodmap
                    | TSKind::methodmap_property
                    | TSKind::methodmap_property_alias
                    | TSKind::methodmap_property_native
                    | TSKind::methodmap_property_method
                    | TSKind::methodmap_property_getter
                    | TSKind::methodmap_property_setter
                    | TSKind::methodmap_native
                    | TSKind::methodmap_native_constructor
                    | TSKind::methodmap_native_destructor
                    | TSKind::methodmap_method
                    | TSKind::methodmap_method_constructor
                    | TSKind::methodmap_method_destructor
                    | TSKind::function_declaration
                    | TSKind::r#enum
                    | TSKind::enum_entries
                    | TSKind::parameter_declarations
                    | TSKind::block
                    | TSKind::for_statement
                    | TSKind::condition_statement
                    | TSKind::do_while_statement
                    | TSKind::switch_statement
                    | TSKind::switch_case
                    | TSKind::while_statement
                    | TSKind::enum_struct
                    | TSKind::enum_struct_method
                    | TSKind::typedef
                    | TSKind::typedef_expression
                    | TSKind::typeset
                    | TSKind::functag
                    | TSKind::funcenum
                    | TSKind::funcenum_member
            )
        });
        AstIdMap { arena, map }
    }

    pub fn ast_id_of(&self, node: &tree_sitter::Node) -> AstId {
        match self.map.get(&NodePtr::from(node)) {
            Some(id) => *id,
            None => {
                for (k, v) in self.map.iter() {
                    log::error!("k: {:?}, v: {:?}", k, v)
                }
                panic!("Failed to find: {:?}", NodePtr::from(node))
            }
        }
    }

    pub(crate) fn get_raw(&self, id: AstId) -> NodePtr {
        self.arena[id.raw]
    }
}

fn bdfs(node: &tree_sitter::Node, f: &mut impl FnMut(tree_sitter::Node) -> bool) {
    let cursor = &mut node.walk();
    let mut nodes = vec![];
    for child in node.children(cursor) {
        if f(child) {
            nodes.push(child);
        }
    }
    for child in nodes {
        bdfs(&child, f);
    }
}

impl Index<AstId> for AstIdMap {
    type Output = NodePtr;
    fn index(&self, index: AstId) -> &Self::Output {
        &self.arena[index.raw]
    }
}
