use std::ops::Index;
use std::sync::Arc;

use base_db::Tree;
use fxhash::FxHashMap;
use la_arena::{Arena, Idx};
use vfs::FileId;

use crate::DefDatabase;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodePtr {
    start_byte: usize,
    end_byte: usize,
}

impl From<&tree_sitter::Node<'_>> for NodePtr {
    fn from(node: &tree_sitter::Node<'_>) -> Self {
        NodePtr {
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
        }
    }
}

impl NodePtr {
    pub fn to_node<'a>(&'a self, tree: &'a Tree) -> tree_sitter::Node {
        let mut node = tree.root_node();
        loop {
            if node.start_byte() == self.start_byte && node.end_byte() == self.end_byte {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstIdMap {
    arena: Arena<NodePtr>,
    map: FxHashMap<NodePtr, AstId>,
}

impl AstIdMap {
    pub fn ast_id_of(&self, node: &tree_sitter::Node) -> AstId {
        for (k, v) in self.map.iter() {
            log::info!("k: {:?}, v: {:?}", k, v)
        }
        log::info!("Looking for: {:?}", NodePtr::from(node));
        self.map[&NodePtr::from(node)]
    }
}

impl AstIdMap {
    pub fn from_tree(db: &dyn DefDatabase, file_id: FileId) -> Arc<Self> {
        let mut arena = Arena::default();
        let mut map = FxHashMap::default();
        let tree = db.parse(file_id);
        let mut cursor = tree.root_node().walk();
        for node in tree.root_node().children(&mut cursor) {
            if !matches!(
                node.kind(),
                "function_declaration" | "global_variable_declaration"
            ) {
                continue;
            }
            let node_ptr = NodePtr::from(&node);
            let ast_id = arena.alloc(node_ptr);
            map.insert(node_ptr, AstId { raw: ast_id });
        }
        Arc::new(AstIdMap { arena, map })
    }
}

impl Index<AstId> for AstIdMap {
    type Output = NodePtr;
    fn index(&self, index: AstId) -> &Self::Output {
        &self.arena[index.raw]
    }
}
