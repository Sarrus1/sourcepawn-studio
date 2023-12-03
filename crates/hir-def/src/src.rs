//! Utilities for mapping between hir IDs and the surface syntax.

use base_db::Tree;

use crate::{item_tree::ItemTreeNode, BlockLoc, DefDatabase, InFile, ItemTreeId};

pub trait HasSource<'tree> {
    fn source(&self, db: &dyn DefDatabase, tree: &'tree Tree) -> InFile<tree_sitter::Node<'tree>>;
}

impl<'tree, N: ItemTreeNode> HasSource<'tree> for ItemTreeId<N> {
    fn source(&self, db: &dyn DefDatabase, tree: &'tree Tree) -> InFile<tree_sitter::Node<'tree>> {
        let item_tree = self.item_tree(db);
        let ast_id_map = db.ast_id_map(self.file_id());
        let item = &item_tree[*self];
        let node_ptr = ast_id_map.get_raw(item.ast_id());
        let node: tree_sitter::Node<'tree> = node_ptr.to_node(tree);
        InFile::new(self.file_id(), node)
    }
}

impl<'tree> HasSource<'tree> for BlockLoc {
    fn source(&self, db: &dyn DefDatabase, tree: &'tree Tree) -> InFile<tree_sitter::Node<'tree>> {
        let ast_id_map = db.ast_id_map(self.file_id);
        let node_ptr = ast_id_map.get_raw(self.ast_id);
        let node: tree_sitter::Node<'tree> = node_ptr.to_node(tree);
        InFile::new(self.file_id, node)
    }
}
