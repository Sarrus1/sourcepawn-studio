use core::hash::Hash;
use item_tree::{AstId, Function, ItemTree, ItemTreeNode, Variable};
use la_arena::Idx;
use std::{hash::Hasher, sync::Arc};
use vfs::FileId;

mod ast_id_map;
pub mod db;
mod item_tree;

pub use ast_id_map::NodePtr;
pub use db::DefDatabase;
pub use item_tree::FileItem;

trait Intern {
    type ID;
    fn intern(self, db: &dyn db::DefDatabase) -> Self::ID;
}

pub trait Lookup {
    type Data;
    fn lookup(&self, db: &dyn db::DefDatabase) -> Self::Data;
}

macro_rules! impl_intern_key {
    ($name:ident) => {
        impl salsa::InternKey for $name {
            fn from_intern_id(v: salsa::InternId) -> Self {
                $name(v)
            }
            fn as_intern_id(&self) -> salsa::InternId {
                self.0
            }
        }
    };
}

macro_rules! impl_intern {
    ($id:ident, $loc:ident, $intern:ident, $lookup:ident) => {
        impl_intern_key!($id);

        impl Intern for $loc {
            type ID = $id;
            fn intern(self, db: &dyn db::DefDatabase) -> $id {
                db.$intern(self)
            }
        }

        impl Lookup for $id {
            type Data = $loc;
            fn lookup(&self, db: &dyn db::DefDatabase) -> $loc {
                db.$lookup(*self)
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(salsa::InternId);
type FunctionLoc = ItemTreeId<Function>;
impl_intern!(
    FunctionId,
    FunctionLoc,
    intern_function,
    lookup_intern_function
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct BlockId(salsa::InternId);
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct BlockLoc {
    ast_id: AstId,
    file_id: FileId,
}
impl_intern!(BlockId, BlockLoc, intern_block, lookup_intern_block);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariableId(salsa::InternId);
type VariableLoc = ItemTreeId<Variable>;
impl_intern!(
    VariableId,
    VariableLoc,
    intern_variable,
    lookup_intern_variable
);

/// Defs which can be visible at the global scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileDefId {
    FunctionId(FunctionId),
    VariableId(VariableId),
}

impl From<FunctionId> for FileDefId {
    fn from(it: FunctionId) -> FileDefId {
        FileDefId::FunctionId(it)
    }
}

/// Identifies a particular [`ItemTree`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct TreeId {
    file: FileId,
}

impl TreeId {
    pub(crate) fn new(file: FileId) -> Self {
        Self { file }
    }

    pub(crate) fn item_tree(&self, db: &dyn DefDatabase) -> Arc<ItemTree> {
        db.file_item_tree(self.file)
    }

    pub(crate) fn file_id(self) -> FileId {
        self.file
    }
}

#[derive(Debug)]
pub struct ItemTreeId<N: ItemTreeNode> {
    tree: TreeId,
    pub value: Idx<N>,
}

impl<N: ItemTreeNode> ItemTreeId<N> {
    pub fn new(tree: TreeId, idx: Idx<N>) -> Self {
        Self { tree, value: idx }
    }

    pub fn file_id(self) -> FileId {
        self.tree.file
    }

    pub fn tree_id(self) -> TreeId {
        self.tree
    }

    pub fn item_tree(self, db: &dyn DefDatabase) -> Arc<ItemTree> {
        self.tree.item_tree(db)
    }
}

impl<N: ItemTreeNode> Copy for ItemTreeId<N> {}
impl<N: ItemTreeNode> Clone for ItemTreeId<N> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<N: ItemTreeNode> PartialEq for ItemTreeId<N> {
    fn eq(&self, other: &Self) -> bool {
        self.tree == other.tree && self.value == other.value
    }
}

impl<N: ItemTreeNode> Eq for ItemTreeId<N> {}

impl<N: ItemTreeNode> Hash for ItemTreeId<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tree.hash(state);
        self.value.hash(state);
    }
}
