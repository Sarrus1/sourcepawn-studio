use bitflags::bitflags;
use core::hash::Hash;
use la_arena::{Arena, Idx, IdxRange};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use smallvec::SmallVec;
use smol_str::SmolStr;
use std::fmt;
use std::ops::Index;
use std::sync::Arc;
use syntax::TSKind;
use vfs::FileId;

pub use crate::ast_id_map::AstId;
use crate::{db::DefDatabase, hir::type_ref::TypeRef, src::HasSource, BlockId, ItemTreeId, Lookup};

use self::lower::Ctx;

mod lower;
mod pretty;

pub use pretty::print_item_tree;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum FunctionKind {
    Def,
    Forward,
    Native,
}

impl FunctionKind {
    pub fn from_node(node: &tree_sitter::Node) -> Self {
        if let Some(kind_node) = node.child_by_field_name("kind") {
            for child in kind_node.children(&mut kind_node.walk()) {
                match TSKind::from(child) {
                    TSKind::anon_forward => return FunctionKind::Forward,
                    TSKind::anon_native => return FunctionKind::Native,
                    _ => (),
                }
            }
        }

        FunctionKind::Def
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RawVisibilityId: u32 {
        const PUBLIC = 1 << 0;
        const STOCK = 1 << 1;
        const STATIC = 1 << 2;
        const NONE = 1 << 3;
    }
}

impl fmt::Debug for RawVisibilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_tuple("RawVisibilityId");

        if self.contains(Self::PUBLIC) {
            f.field(&"public");
        }
        if self.contains(Self::STOCK) {
            f.field(&"stock");
        }
        if self.contains(Self::STATIC) {
            f.field(&"static");
        }
        f.finish()
    }
}

impl std::fmt::Display for RawVisibilityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        if self.contains(Self::PUBLIC) {
            s.push_str("public ");
        }
        if self.contains(Self::STOCK) {
            s.push_str("stock ");
        }
        if self.contains(Self::STATIC) {
            s.push_str("static ");
        }

        write!(f, "{}", s.trim_end())
    }
}

impl RawVisibilityId {
    pub fn from_node(node: &tree_sitter::Node) -> Self {
        let mut visibility = RawVisibilityId::NONE;
        let Some(visibility_node) = node.child_by_field_name("visibility") else {
            return visibility;
        };
        for child in visibility_node.children(&mut visibility_node.walk()) {
            match TSKind::from(child) {
                TSKind::anon_public => visibility |= RawVisibilityId::PUBLIC,
                TSKind::anon_stock => visibility |= RawVisibilityId::STOCK,
                TSKind::anon_static => visibility |= RawVisibilityId::STATIC,
                _ => log::error!("Unexpected child of visibility: {:?}", child),
            }
        }
        visibility
    }
}

/// The item tree of a source file.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct ItemTree {
    top_level: SmallVec<[FileItem; 1]>,
    data: Option<Box<ItemTreeData>>,
}

impl ItemTree {
    pub fn file_item_tree_query(db: &dyn DefDatabase, file_id: FileId) -> Arc<Self> {
        let mut ctx = Ctx::new(db, file_id);

        ctx.lower();
        ctx.finish()
    }

    pub fn block_item_tree_query(db: &dyn DefDatabase, block: BlockId) -> Arc<Self> {
        let loc = block.lookup(db);
        let tree = db.parse(loc.file_id);
        let block_node = loc.source(db, &tree);
        let source = db.preprocessed_text(loc.file_id);
        let ast_id_map = db.ast_id_map(loc.file_id);
        let mut item_tree = ItemTree::default();
        for child in block_node.value.children(&mut block_node.value.walk()) {
            match TSKind::from(child) {
                TSKind::variable_declaration_statement => {
                    let type_ref = TypeRef::from_returntype_node(&child, "type", &source);
                    for sub_child in child.children(&mut child.walk()) {
                        if matches!(
                            TSKind::from(sub_child),
                            TSKind::variable_declaration | TSKind::dynamic_array_declaration
                        ) {
                            if let Some(name_node) = sub_child.child_by_field_name("name") {
                                let res = Variable {
                                    name: Name::from(
                                        name_node.utf8_text(source.as_bytes()).unwrap(),
                                    ),
                                    visibility: RawVisibilityId::NONE,
                                    type_ref: type_ref.clone(),
                                    ast_id: ast_id_map.ast_id_of(&sub_child),
                                };
                                let id = item_tree.data_mut().variables.alloc(res);
                                item_tree.top_level.push(FileItem::Variable(id));
                            }
                        }
                    }
                }
                TSKind::old_variable_declaration_statement
                | TSKind::old_for_loop_variable_declaration_statement => {
                    for sub_child in child
                        .children(&mut child.walk())
                        .filter(|n| TSKind::from(n) == TSKind::old_variable_declaration)
                    {
                        let type_ref = TypeRef::from_returntype_node(&sub_child, "type", &source);
                        if let Some(name_node) = sub_child.child_by_field_name("name") {
                            let res = Variable {
                                name: Name::from(name_node.utf8_text(source.as_bytes()).unwrap()),
                                visibility: RawVisibilityId::NONE,
                                type_ref: type_ref.clone(),
                                ast_id: ast_id_map.ast_id_of(&sub_child),
                            };
                            let id = item_tree.data_mut().variables.alloc(res);
                            item_tree.top_level.push(FileItem::Variable(id));
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
    macros: Arena<Macro>,
    enum_structs: Arena<EnumStruct>,
    fields: Arena<Field>,
    methodmaps: Arena<Methodmap>,
    properties: Arena<Property>,
    params: Arena<Param>,
    enums: Arena<Enum>,
    typedefs: Arena<Typedef>,
    typesets: Arena<Typeset>,
    functags: Arena<Functag>,
    funcenums: Arena<Funcenum>,
    variants: Arena<Variant>,
    structs: Arena<Struct>,
    struct_fields: Arena<StructField>,
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

impl Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NameVisitor;

        impl Visitor<'_> for NameVisitor {
            type Value = Name;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string for SmolStr")
            }

            fn visit_str<E>(self, value: &str) -> Result<Name, E>
            where
                E: de::Error,
            {
                Ok(Name(SmolStr::new(value)))
            }
        }

        deserializer.deserialize_str(NameVisitor)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Macro {
    pub name: Name,
    // pub params: IdxRange<Param>,
    pub ast_id: AstId,
    pub deprecated: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Variable {
    pub name: Name,
    pub visibility: RawVisibilityId,
    pub type_ref: Option<TypeRef>,
    pub ast_id: AstId,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SpecialMethod {
    Constructor,
    Destructor,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Function {
    pub name: Name,
    pub kind: FunctionKind,
    pub visibility: RawVisibilityId,
    pub params: IdxRange<Param>,
    pub special: Option<SpecialMethod>,
    pub ret_type: Option<TypeRef>,
    pub ast_id: AstId,
    pub deprecated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub has_default: bool,
    pub is_rest: bool,
    pub is_const: bool,
    pub type_ref: Option<TypeRef>,
    pub ast_id: AstId,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Typedef {
    pub name: Option<Name>,
    pub params: IdxRange<Param>,
    pub type_ref: TypeRef,
    pub ast_id: AstId,
    pub deprecated: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Typeset {
    pub name: Name,
    pub typedefs: IdxRange<Typedef>,
    pub ast_id: AstId,
    pub deprecated: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Functag {
    pub name: Option<Name>,
    pub params: IdxRange<Param>,
    pub type_ref: Option<TypeRef>,
    pub ast_id: AstId,
    pub deprecated: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Funcenum {
    pub name: Name,
    pub functags: IdxRange<Functag>,
    pub ast_id: AstId,
    pub deprecated: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EnumStructItemId {
    Method(Idx<Function>),
    Field(Idx<Field>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EnumStruct {
    pub name: Name,
    pub items: Box<[EnumStructItemId]>,
    pub ast_id: AstId,
    pub deprecated: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Struct {
    pub name: Name,
    pub fields: IdxRange<StructField>,
    pub ast_id: AstId,
    pub deprecated: bool,
}

/// A single field of a struct
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    pub name: Name,
    pub const_: bool,
    pub type_ref: TypeRef,
    pub ast_id: AstId,
    pub deprecated: bool,
}

/// A single field of an enum struct
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: Name,
    pub type_ref: TypeRef,
    pub ast_id: AstId,
    pub deprecated: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MethodmapItemId {
    Method(Idx<Function>),
    Property(Idx<Property>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    pub name: Name,
    pub getters_setters: IdxRange<Function>,
    pub type_ref: TypeRef,
    pub ast_id: AstId,
    pub deprecated: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Methodmap {
    pub name: Name,
    pub items: Box<[MethodmapItemId]>,
    pub inherits: Option<Name>,
    pub nullable: bool,
    pub ast_id: AstId,
    pub deprecated: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Enum {
    pub name: Name,
    pub variants: IdxRange<Variant>,
    pub ast_id: AstId,
    pub deprecated: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Variant {
    pub name: Name,
    pub ast_id: AstId,
    pub deprecated: bool,
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
    Macro macros,
    EnumStruct enum_structs,
    Methodmap methodmaps,
    Property properties,
    Enum enums,
    Variant variants,
    Typedef typedefs,
    Typeset typesets,
    Functag functags,
    Funcenum funcenums,
    Struct structs
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
    params: Param,
    struct_fields: StructField
}

impl<N: ItemTreeNode> Index<ItemTreeId<N>> for ItemTree {
    type Output = N;
    fn index(&self, id: ItemTreeId<N>) -> &N {
        N::lookup(self, id.value)
    }
}
