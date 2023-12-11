use core::hash::Hash;
use la_arena::{Arena, Idx};
use smallvec::SmallVec;
use smol_str::SmolStr;
use std::ops::Index;
use std::sync::Arc;
use syntax::TSKind;
use vfs::FileId;

pub use crate::ast_id_map::{AstId, NodePtr};
use crate::{db::DefDatabase, src::HasSource, BlockId, ItemTreeId, Lookup};

use self::pretty::print_item_tree;

mod pretty;

/// The item tree of a source file.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct ItemTree {
    // attrs: FxHashMap<AttrOwner, RawAttrs>,
    top_level: SmallVec<[FileItem; 1]>,
    data: Option<Box<ItemTreeData>>,
}

impl ItemTree {
    pub fn file_item_tree_query(db: &dyn DefDatabase, file_id: FileId) -> Arc<Self> {
        let mut item_tree = ItemTree::default();
        let tree = db.parse(file_id);
        let root_node = tree.root_node();
        let source = db.file_text(file_id);
        let source = source.as_bytes();
        let ast_id_map = db.ast_id_map(file_id);
        for child in root_node.children(&mut root_node.walk()) {
            match TSKind::from(child) {
                TSKind::sym_function_declaration => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let res = Function {
                            name: Name::from(name_node.utf8_text(source).unwrap()),
                            ast_id: ast_id_map.ast_id_of(&child),
                        };
                        let id = item_tree.data_mut().functions.alloc(res);
                        item_tree.top_level.push(FileItem::Function(id));
                    }
                }
                TSKind::sym_global_variable_declaration => {
                    let mut cursor = child.walk();
                    for sub_child in child.children(&mut cursor) {
                        if TSKind::from(sub_child) == TSKind::sym_variable_declaration {
                            if let Some(name_node) = sub_child.child_by_field_name("name") {
                                let res = Variable {
                                    name: Name::from(name_node.utf8_text(source).unwrap()),
                                    ast_id: ast_id_map.ast_id_of(&sub_child),
                                };
                                let id = item_tree.data_mut().variables.alloc(res);
                                item_tree.top_level.push(FileItem::Variable(id));
                            }
                        }
                    }
                }
                _ => (),
            }
        }
        print_item_tree(db, &item_tree);
        Arc::new(item_tree)
    }

    pub fn block_item_tree_query(db: &dyn DefDatabase, block: BlockId) -> Arc<Self> {
        let loc = block.lookup(db);
        let tree = db.parse(loc.file_id);
        let block_node = loc.source(db, &tree);
        let source = db.file_text(loc.file_id);
        let ast_id_map = db.ast_id_map(loc.file_id);
        let mut item_tree = ItemTree::default();
        for child in block_node.value.children(&mut block_node.value.walk()) {
            match TSKind::from(child) {
                TSKind::sym_variable_declaration_statement => {
                    for sub_child in child.children(&mut child.walk()) {
                        if TSKind::from(sub_child) == TSKind::sym_variable_declaration {
                            if let Some(name_node) = sub_child.child_by_field_name("name") {
                                let res = Variable {
                                    name: Name::from(
                                        name_node.utf8_text(source.as_bytes()).unwrap(),
                                    ),
                                    ast_id: ast_id_map.ast_id_of(&sub_child),
                                };
                                let id = item_tree.data_mut().variables.alloc(res);
                                item_tree.top_level.push(FileItem::Variable(id));
                            }
                        }
                    }
                }
                _ => log::error!("Unexpected child of block: {:?}", child),
            }
        }
        Arc::new(item_tree)
    }

    /// Returns an iterator over all items located at the top level of the `HirFileId` this
    /// `ItemTree` was created from.
    pub fn top_level_items(&self) -> &[FileItem] {
        &self.top_level
    }

    fn data(&self) -> &ItemTreeData {
        self.data
            .as_ref()
            .expect("attempted to access data of empty ItemTree")
    }

    fn data_mut(&mut self) -> &mut ItemTreeData {
        self.data.get_or_insert_with(Box::default)
    }
}

#[derive(Default, Debug, Eq, PartialEq)]
struct ItemTreeData {
    functions: Arena<Function>,
    variables: Arena<Variable>,
    // params: Arena<Param>,
}

/// `Name` is a wrapper around string, which is used in hir for both references
/// and declarations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Name(SmolStr);

impl From<&str> for Name {
    fn from(value: &str) -> Self {
        Name(SmolStr::from(value))
    }
}

impl Into<String> for Name {
    fn into(self) -> String {
        self.0.into()
    }
}

impl Name {
    pub fn from_node(node: &tree_sitter::Node, source: &str) -> Self {
        Self::from(
            node.utf8_text(source.as_bytes())
                .expect("Failed to get utf8 text"),
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Variable {
    pub name: Name,
    // pub visibility: RawVisibilityId,
    // pub type_ref: Interned<TypeRef>,
    pub ast_id: AstId,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Function {
    pub name: Name,
    // pub visibility: RawVisibilityId,
    // pub params: IdxRange<Param>,
    // pub ret_type: Interned<TypeRef>,
    pub ast_id: AstId,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Block {
    pub ast_id: AstId,
}

/// Trait implemented by all item nodes in the item tree.
pub trait ItemTreeNode: Clone {
    fn ast_id(&self) -> AstId;

    /// Looks up an instance of `Self` in an item tree.
    fn lookup(tree: &ItemTree, index: Idx<Self>) -> &Self;

    /// Downcasts a `ModItem` to a `FileItemTreeId` specific to this type.
    fn id_from_mod_item(mod_item: FileItem) -> Option<Idx<Self>>;

    /// Upcasts a `FileItemTreeId` to a generic `ModItem`.
    fn id_to_mod_item(id: Idx<Self>) -> FileItem;
}

macro_rules! mod_items {
    ( $( $typ:ident $fld:ident ),+ $(,)? ) => {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub enum FileItem {
            $(
                $typ(Idx<$typ>),
            )+
        }

        $(
            impl From<Idx<$typ>> for FileItem {
                fn from(id: Idx<$typ>) -> FileItem {
                    FileItem::$typ(id)
                }
            }
        )+

        $(
            impl ItemTreeNode for $typ {
                fn ast_id(&self) -> AstId {
                    self.ast_id
                }

                fn lookup(tree: &ItemTree, index: Idx<Self>) -> &Self {
                    &tree.data().$fld[index]
                }

                fn id_from_mod_item(mod_item: FileItem) -> Option<Idx<Self>> {
                    match mod_item {
                        FileItem::$typ(id) => Some(id),
                        _ => None,
                    }
                }

                fn id_to_mod_item(id: Idx<Self>) -> FileItem {
                    FileItem::$typ(id)
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
    Function functions,
    Variable variables,
}

impl<N: ItemTreeNode> Index<ItemTreeId<N>> for ItemTree {
    type Output = N;
    fn index(&self, id: ItemTreeId<N>) -> &N {
        N::lookup(self, id.value)
    }
}
