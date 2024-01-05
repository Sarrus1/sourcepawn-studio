use core::hash::Hash;
use la_arena::{Arena, Idx, IdxRange, RawIdx};
use smallvec::SmallVec;
use smol_str::SmolStr;
use std::fmt;
use std::ops::Index;
use std::sync::Arc;
use syntax::TSKind;
use vfs::FileId;

pub use crate::ast_id_map::{AstId, NodePtr};
use crate::{db::DefDatabase, hir::type_ref::TypeRef, src::HasSource, BlockId, ItemTreeId, Lookup};

mod pretty;

/// The item tree of a source file.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct ItemTree {
    // attrs: FxHashMap<AttrOwner, RawAttrs>,
    top_level: SmallVec<[FileItem; 1]>,
    data: Option<Box<ItemTreeData>>,
}

fn function_return_type(node: &tree_sitter::Node, source: &str) -> Option<TypeRef> {
    let ret_type_node = node.child_by_field_name("returnType")?;
    for child in ret_type_node.children(&mut ret_type_node.walk()) {
        match TSKind::from(child) {
            TSKind::r#type => return TypeRef::from_node(&child, source),
            TSKind::old_type => {
                for sub_child in child.children(&mut child.walk()) {
                    match TSKind::from(sub_child) {
                        TSKind::old_builtin_type | TSKind::identifier | TSKind::any_type => {
                            return Some(TypeRef::OldString)
                        }
                        _ => (),
                    }
                }
                return TypeRef::from_node(&child, source);
            }
            _ => (),
        }
    }
    None
}

impl ItemTree {
    fn next_field_idx(&self) -> Idx<Field> {
        Idx::from_raw(RawIdx::from(
            self.data
                .as_ref()
                .map_or(0, |data| data.fields.len() as u32),
        ))
    }
    pub fn file_item_tree_query(db: &dyn DefDatabase, file_id: FileId) -> Arc<Self> {
        let mut item_tree = ItemTree::default();
        let tree = db.parse(file_id);
        let root_node = tree.root_node();
        let source = db.file_text(file_id);
        let ast_id_map = db.ast_id_map(file_id);
        for child in root_node.children(&mut root_node.walk()) {
            match TSKind::from(child) {
                TSKind::function_definition => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let res = Function {
                            name: Name::from(name_node.utf8_text(source.as_bytes()).unwrap()),
                            ret_type: function_return_type(&child, &source),
                            ast_id: ast_id_map.ast_id_of(&child),
                        };
                        let id = item_tree.data_mut().functions.alloc(res);
                        item_tree.top_level.push(FileItem::Function(id));
                    }
                }
                TSKind::global_variable_declaration => {
                    let type_ref = if let Some(type_node) = child.child_by_field_name("type") {
                        TypeRef::from_node(&type_node, &source)
                    } else {
                        None
                    };
                    for sub_child in child.children(&mut child.walk()) {
                        if TSKind::from(sub_child) == TSKind::variable_declaration {
                            if let Some(name_node) = sub_child.child_by_field_name("name") {
                                let res = Variable {
                                    name: Name::from(
                                        name_node.utf8_text(source.as_bytes()).unwrap(),
                                    ),
                                    type_ref: type_ref.clone(),
                                    ast_id: ast_id_map.ast_id_of(&sub_child),
                                };
                                let id = item_tree.data_mut().variables.alloc(res);
                                item_tree.top_level.push(FileItem::Variable(id));
                            }
                        }
                    }
                }
                TSKind::enum_struct => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let start = item_tree.next_field_idx();
                        child
                            .children(&mut child.walk())
                            .filter(|e| TSKind::from(e) == TSKind::enum_struct_field)
                            .for_each(|e| {
                                let Some(field_name_node) = e.child_by_field_name("name") else {
                                    return;
                                };
                                let Some(field_type_node) = e.child_by_field_name("type") else {
                                    return;
                                };
                                let res = Field {
                                    name: Name::from(
                                        field_name_node.utf8_text(source.as_bytes()).unwrap(),
                                    ),
                                    type_ref: TypeRef::from_node(&field_type_node, &source)
                                        .unwrap(),
                                    ast_id: ast_id_map.ast_id_of(&e),
                                };
                                item_tree.data_mut().fields.alloc(res);
                            });
                        let end = item_tree.next_field_idx();
                        let res = EnumStruct {
                            name: Name::from(name_node.utf8_text(source.as_bytes()).unwrap()),
                            fields: IdxRange::new(start..end),
                            ast_id: ast_id_map.ast_id_of(&child),
                        };
                        let id = item_tree.data_mut().enum_structs.alloc(res);
                        item_tree.top_level.push(FileItem::EnumStruct(id));
                    }
                }
                _ => (),
            }
        }
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
                TSKind::variable_declaration_statement => {
                    let type_ref = if let Some(type_node) = child.child_by_field_name("type") {
                        TypeRef::from_node(&type_node, &source)
                    } else {
                        None
                    };
                    for sub_child in child.children(&mut child.walk()) {
                        if TSKind::from(sub_child) == TSKind::variable_declaration {
                            if let Some(name_node) = sub_child.child_by_field_name("name") {
                                let res = Variable {
                                    name: Name::from(
                                        name_node.utf8_text(source.as_bytes()).unwrap(),
                                    ),
                                    type_ref: type_ref.clone(),
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
    enum_structs: Arena<EnumStruct>,
    fields: Arena<Field>,
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

impl From<Name> for String {
    fn from(val: Name) -> Self {
        val.0.into()
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

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Variable {
    pub name: Name,
    // pub visibility: RawVisibilityId,
    pub type_ref: Option<TypeRef>,
    pub ast_id: AstId,
} // TODO: Each variable decl is stored as a separate item, but we should probably group them up ?

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Function {
    pub name: Name,
    // pub visibility: RawVisibilityId,
    // pub params: IdxRange<Param>,
    pub ret_type: Option<TypeRef>,
    pub ast_id: AstId,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EnumStruct {
    pub name: Name,
    pub fields: IdxRange<Field>,
    pub ast_id: AstId,
}

/// A single field of an enum variant or struct
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: Name,
    pub type_ref: TypeRef,
    // pub visibility: RawVisibilityId,
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
    EnumStruct enum_structs,
}

macro_rules! impl_index {
    ( $($fld:ident: $t:ty),+ $(,)? ) => {
        $(
            impl Index<Idx<$t>> for ItemTree {
                type Output = $t;

                fn index(&self, index: Idx<$t>) -> &Self::Output {
                    &self.data().$fld[index]
                }
            }
        )+
    };
}

impl_index! {
    fields: Field,
}

impl<N: ItemTreeNode> Index<ItemTreeId<N>> for ItemTree {
    type Output = N;
    fn index(&self, id: ItemTreeId<N>) -> &N {
        N::lookup(self, id.value)
    }
}
