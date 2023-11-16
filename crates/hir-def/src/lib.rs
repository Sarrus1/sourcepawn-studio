use core::{fmt, hash::Hash};
use db::DefDatabase;
use fxhash::FxHashMap;
use la_arena::{Arena, Idx};
use smallvec::SmallVec;
use smol_str::SmolStr;
use std::{hash::Hasher, marker::PhantomData, sync::Arc};
use vfs::FileId;

mod db;
mod item_tree;

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

        impl<'tree> Intern for $loc<'tree> {
            type ID = $id;
            fn intern(self, db: &dyn db::DefDatabase) -> $id {
                db.$intern(self)
            }
        }

        // impl Lookup for $id {
        //     type Data<'tree> = $loc<'tree>;
        //     fn lookup(&self, db: &dyn db::DefDatabase) -> $loc {
        //         db.$lookup(*self)
        //     }
        // }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(salsa::InternId);
type FunctionLoc<'tree> = ItemTreeId<'tree, Function<'tree>>;
impl_intern!(
    FunctionId,
    FunctionLoc,
    intern_function,
    lookup_intern_function
);

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
pub struct ItemTreeId<'tree, N: ItemTreeNode<'tree>> {
    tree: TreeId,
    pub value: FileItemTreeId<'tree, N>,
}

impl<'tree, N: ItemTreeNode<'tree>> ItemTreeId<'tree, N> {
    // pub fn new(tree: TreeId, idx: FileItemTreeId<N>) -> Self {
    //     Self { tree, value: idx }
    // }

    // pub fn file_id(self) -> FileId {
    //     self.tree.file
    // }

    // pub fn tree_id(self) -> TreeId {
    //     self.tree
    // }

    pub fn item_tree(self, db: &dyn DefDatabase) -> Arc<ItemTree> {
        self.tree.item_tree(db)
    }
}

impl<'tree, N: ItemTreeNode<'tree>> Clone for ItemTreeId<'tree, N> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'tree, N: ItemTreeNode<'tree>> PartialEq for ItemTreeId<'tree, N> {
    fn eq(&self, other: &Self) -> bool {
        self.tree == other.tree && self.value == other.value
    }
}

impl<'tree, N: ItemTreeNode<'tree>> Eq for ItemTreeId<'tree, N> {}

impl<'tree, N: ItemTreeNode<'tree>> Hash for ItemTreeId<'tree, N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tree.hash(state);
        self.value.hash(state);
    }
}

/// The item tree of a source file.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct ItemTree<'tree> {
    // attrs: FxHashMap<AttrOwner, RawAttrs>,
    top_level: SmallVec<[FileItem<'tree>; 1]>,
    data: Option<Box<ItemTreeData<'tree>>>,
}

impl<'tree> ItemTree<'tree> {
    fn file_item_tree_query(db: &dyn DefDatabase, file_id: FileId) -> Arc<Self> {
        Arc::default()
    }

    fn data(&self) -> &ItemTreeData {
        self.data
            .as_ref()
            .expect("attempted to access data of empty ItemTree")
    }

    fn data_mut(&'tree mut self) -> &mut ItemTreeData {
        self.data.get_or_insert_with(Box::default)
    }
}

#[derive(Default, Debug, Eq, PartialEq)]
struct ItemTreeData<'tree> {
    functions: Arena<Function<'tree>>,
    variables: Arena<Variable<'tree>>,
    // params: Arena<Param>,
}

/// `Name` is a wrapper around string, which is used in hir for both references
/// and declarations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Name(SmolStr);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Variable<'tree> {
    pub name: Name,
    // pub visibility: RawVisibilityId,
    // pub type_ref: Interned<TypeRef>,
    pub ts_node: &'tree tree_sitter::Node<'tree>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Function<'tree> {
    pub name: Name,
    // pub visibility: RawVisibilityId,
    // pub params: IdxRange<Param>,
    // pub ret_type: Interned<TypeRef>,
    pub ts_node: &'tree tree_sitter::Node<'tree>,
}

/// Trait implemented by all item nodes in the item tree.
pub trait ItemTreeNode<'tree>: Clone {
    // fn ast_id(&self) -> FileAstId<tree_sitter::Node>;

    /// Looks up an instance of `Self` in an item tree.
    // fn lookup(tree: &ItemTree, index: Idx<Self>) -> &Self;

    /// Downcasts a `ModItem` to a `FileItemTreeId` specific to this type.
    // fn id_from_mod_item(mod_item: ModItem) -> Option<FileItemTreeId<Self>>;

    /// Upcasts a `FileItemTreeId` to a generic `ModItem`.
    fn id_to_mod_item(id: FileItemTreeId<'tree, Self>) -> FileItem<'tree>;
}

pub struct FileItemTreeId<'tree, N: ItemTreeNode<'tree>> {
    index: Idx<N>,
    _p: PhantomData<&'tree N>,
}

impl<'tree, N: ItemTreeNode<'tree>> Clone for FileItemTreeId<'tree, N> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            _p: PhantomData,
        }
    }
}
impl<'tree, N: ItemTreeNode<'tree>> Copy for FileItemTreeId<'tree, N> {}

impl<'tree, N: ItemTreeNode<'tree>> PartialEq for FileItemTreeId<'tree, N> {
    fn eq(&self, other: &FileItemTreeId<'tree, N>) -> bool {
        self.index == other.index
    }
}
impl<'tree, N: ItemTreeNode<'tree>> Eq for FileItemTreeId<'tree, N> {}

impl<'tree, N: ItemTreeNode<'tree>> Hash for FileItemTreeId<'tree, N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state)
    }
}

impl<'tree, N: ItemTreeNode<'tree>> fmt::Debug for FileItemTreeId<'tree, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.index.fmt(f)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum FileItem<'tree> {
    Function(FileItemTreeId<'tree, Function<'tree>>),
}

impl<'tree> ItemTreeNode<'tree> for Function<'tree> {
    fn id_to_mod_item(id: FileItemTreeId<'tree, Self>) -> FileItem<'tree> {
        FileItem::Function(id)
    }
}

/*
impl<'tree> From<FileItemTreeId<'tree, Function<'tree>>> for FileItem<'tree> {
    fn from(id: FileItemTreeId<'tree, Function>) -> FileItem<'tree> {
        FileItem::Function(id)
    }
}
impl ItemTreeNode for Use {
    type Source = ast::Use;
    fn ast_id(&self) -> FileAstId<Self::Source> {
        self.ast_id
    }
    fn lookup(tree: &ItemTree, index: Idx<Self>) -> &Self {
        &tree.data().uses[index]
    }
    fn id_from_mod_item(mod_item: ModItem) -> Option<FileItemTreeId<Self>> {
        match mod_item {
            ModItem::Use(id) => Some(id),
            _ => None,
        }
    }
    fn id_to_mod_item(id: FileItemTreeId<Self>) -> ModItem {
        ModItem::Use(id)
    }
}
impl Index<Idx<Use>> for ItemTree {
    type Output = Use;
    fn index(&self, index: Idx<Use>) -> &Self::Output {
        &self.data().uses[index]
    }
}
*/

/*
macro_rules! mod_items {
    ( $( $typ:ident in $fld:ident -> $ast:ty ),+ $(,)? ) => {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub enum ModItem {
            $(
                $typ(FileItemTreeId<$typ>),
            )+
        }

        $(
            impl From<FileItemTreeId<$typ>> for ModItem {
                fn from(id: FileItemTreeId<$typ>) -> ModItem {
                    ModItem::$typ(id)
                }
            }
        )+

        $(
            impl ItemTreeNode for $typ {
                type Source = $ast;

                fn ast_id(&self) -> FileAstId<Self::Source> {
                    self.ast_id
                }

                fn lookup(tree: &ItemTree, index: Idx<Self>) -> &Self {
                    &tree.data().$fld[index]
                }

                fn id_from_mod_item(mod_item: ModItem) -> Option<FileItemTreeId<Self>> {
                    match mod_item {
                        ModItem::$typ(id) => Some(id),
                        _ => None,
                    }
                }

                fn id_to_mod_item(id: FileItemTreeId<Self>) -> ModItem {
                    ModItem::$typ(id)
                }
            }

            impl Index<Idx<$typ>> for ItemTree {
                type Output = $typ;

                fn index(&self, index: Idx<$typ>) -> &Self::Output {
                    &self.data().$fld[index]
                }
            }
        )+
    };
}

mod_items! {
    Function in functions -> ast::Fn,
}
*/
