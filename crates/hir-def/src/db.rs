use std::sync::Arc;

use base_db::SourceDatabase;
use fxhash::FxHashMap;
use lazy_static::lazy_static;
use regex::Regex;
use syntax::TSKind;
use tree_sitter::QueryCursor;
use vfs::{AnchoredUrl, FileId};

use crate::{
    ast_id_map::AstIdMap,
    body::{scope::ExprScopes, Body, BodySourceMap},
    data::{EnumStructData, FunctionData},
    infer,
    item_tree::{ItemTree, Name},
    BlockId, BlockLoc, DefWithBodyId, EnumStructId, EnumStructLoc, FileDefId, FileItem, FunctionId,
    FunctionLoc, GlobalId, GlobalLoc, InFile, InferenceResult, Intern, ItemTreeId, Lookup, NodePtr,
    TreeId,
};

#[salsa::query_group(InternDatabaseStorage)]
pub trait InternDatabase: SourceDatabase {
    // region: items
    #[salsa::interned]
    fn intern_function(&'tree self, loc: FunctionLoc) -> FunctionId;
    #[salsa::interned]
    fn intern_enum_struct(&'tree self, loc: EnumStructLoc) -> EnumStructId;
    #[salsa::interned]
    fn intern_variable(&'tree self, loc: GlobalLoc) -> GlobalId;
    #[salsa::interned]
    fn intern_block(&'tree self, loc: BlockLoc) -> BlockId;
    // endregion: items
}

#[salsa::query_group(DefDatabaseStorage)]
pub trait DefDatabase: InternDatabase {
    #[salsa::invoke(ItemTree::file_item_tree_query)]
    fn file_item_tree(&self, file_id: FileId) -> Arc<ItemTree>;

    #[salsa::invoke(ItemTree::block_item_tree_query)]
    fn block_item_tree(&self, block_id: BlockId) -> Arc<ItemTree>;

    #[salsa::invoke(AstIdMap::from_tree)]
    fn ast_id_map(&self, file_id: FileId) -> Arc<AstIdMap>;

    // #[salsa::transparent]
    #[salsa::invoke(DefMap::file_def_map_query)]
    fn file_def_map(&self, file_id: FileId) -> Arc<DefMap>;

    #[salsa::invoke(DefMap::block_def_map_query)]
    fn block_def_map(&self, block_id: BlockId) -> Arc<DefMap>;

    #[salsa::invoke(Body::body_with_source_map_query)]
    fn body_with_source_map(&self, def: DefWithBodyId) -> (Arc<Body>, Arc<BodySourceMap>);

    #[salsa::invoke(Body::body_query)]
    fn body(&self, def: DefWithBodyId) -> Arc<Body>;

    #[salsa::invoke(ExprScopes::expr_scopes_query)]
    fn expr_scopes(&self, def: DefWithBodyId, file_id: FileId) -> Arc<ExprScopes>;

    // region: data
    #[salsa::invoke(FunctionData::function_data_query)]
    fn function_data(&self, id: FunctionId) -> Arc<FunctionData>;

    #[salsa::invoke(EnumStructData::enum_struct_data_query)]
    fn enum_struct_data(&self, id: EnumStructId) -> Arc<EnumStructData>;
    // endregion: data

    #[salsa::invoke(file_includes_query)]
    fn file_includes(&self, file_id: FileId) -> (Arc<Vec<Include>>, Arc<Vec<UnresolvedInclude>>);

    // region: infer
    #[salsa::invoke(infer::infer_query)]
    fn infer(&self, def: DefWithBodyId) -> Arc<InferenceResult>;
    // endregion: infer
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IncludeType {
    /// #include <foo>
    Include,

    /// #tryinclude <foo>
    TryInclude,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IncludeKind {
    /// #include <foo>
    Chevrons,

    /// #include "foo"
    Quotes,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Include {
    id: FileId,
    kind: IncludeKind,
    type_: IncludeType,
}

impl Include {
    pub fn file_id(&self) -> FileId {
        self.id
    }

    pub fn kind(&self) -> IncludeKind {
        self.kind
    }

    pub fn type_(&self) -> IncludeType {
        self.type_
    }
}

pub(crate) fn file_includes_query(
    db: &dyn DefDatabase,
    file_id: FileId,
) -> (Arc<Vec<Include>>, Arc<Vec<UnresolvedInclude>>) {
    let tree = db.parse(file_id);
    // FIXME: We should use the preprocessed text here. Some includes might be removed by the preprocessor.
    let source = db.file_text(file_id);

    lazy_static! {
        static ref INCLUDE_QUERY: tree_sitter::Query = tree_sitter::Query::new(
            tree_sitter_sourcepawn::language(),
            "(preproc_include) @include
             (preproc_tryinclude) @tryinclude"
        )
        .expect("Could not build includes query.");
    }

    let mut res = vec![];
    let mut cursor = QueryCursor::new();
    let matches = cursor.captures(&INCLUDE_QUERY, tree.root_node(), source.as_bytes());

    let mut unresolved = vec![];
    for (match_, _) in matches {
        res.extend(match_.captures.iter().filter_map(|c| {
            let node = c.node.child_by_field_name("path")?;
            let text = node.utf8_text(source.as_bytes()).ok()?;
            lazy_static! {
                static ref RE_CHEVRON: Regex = Regex::new(r"<([^>]+)>").unwrap();
                static ref RE_QUOTE: Regex = Regex::new("\"([^>]+)\"").unwrap();
            }
            let type_ = match TSKind::from(&c.node) {
                TSKind::preproc_include => IncludeType::Include,
                TSKind::preproc_tryinclude => IncludeType::TryInclude,
                _ => unreachable!(),
            };
            let text = match TSKind::from(node) {
                TSKind::system_lib_string => RE_CHEVRON.captures(text)?.get(1)?.as_str(),
                TSKind::string_literal => {
                    let text = RE_QUOTE.captures(text)?.get(1)?.as_str();
                    // try to resolve path relative to the referencing file.
                    if let Some(file_id) = db.resolve_path(AnchoredUrl::new(file_id, text)) {
                        return Some(Include {
                            id: file_id,
                            kind: IncludeKind::Quotes,
                            type_,
                        });
                    }
                    text
                }
                _ => unreachable!(),
            };
            // TODO: resolve the include
            if type_ == IncludeType::Include {
                // TODO: Add setting for optional diagnostic for tryinclude.
                unresolved.push(UnresolvedInclude {
                    expr: InFile::new(file_id, NodePtr::from(&node)),
                    path: text.to_string(),
                });
            }
            None
        }));
    }

    (Arc::new(res), Arc::new(unresolved))
}

pub fn infer_include_ext(path: &str) -> String {
    if path.ends_with(".sp") || path.ends_with(".inc") {
        path.to_string()
    } else {
        format!("{}.inc", path)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UnresolvedInclude {
    pub expr: InFile<NodePtr>,
    pub path: String,
}

/// For `DefMap`s computed for a block expression, this stores its location in the parent map.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct BlockInfo {
    /// The `BlockId` this `DefMap` was created from.
    block: BlockId,
    /// The containing file.
    parent: FileId,
}

// FIXME: DefMap should not be used as a scope. It should be used to map ids to defs.
// It should be useless to have a DefMap for a block, as they do not define functions etc.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct DefMap {
    values: FxHashMap<Name, FileDefId>,
    declarations: Vec<FileDefId>,
    /// When this is a block def map, this will hold the block id of the block and module that
    /// contains this block.
    block: Option<BlockInfo>,
}

impl DefMap {
    pub fn file_def_map_query(db: &dyn DefDatabase, file_id: FileId) -> Arc<Self> {
        let mut res = DefMap::default();
        let item_tree = db.file_item_tree(file_id);
        let tree_id = TreeId::new(file_id, None);
        for item in item_tree.top_level_items() {
            match item {
                FileItem::Function(id) => {
                    let func = &item_tree[*id];
                    let fn_id = FunctionLoc {
                        container: file_id.into(),
                        id: ItemTreeId {
                            tree: tree_id,
                            value: *id,
                        },
                    }
                    .intern(db);
                    res.declare(func.name.clone(), FileDefId::FunctionId(fn_id));
                }
                FileItem::Variable(id) => {
                    let var = &item_tree[*id];
                    let var_id = GlobalLoc {
                        tree: TreeId::new(file_id, None),
                        value: *id,
                    }
                    .intern(db);
                    res.declare(var.name.clone(), FileDefId::GlobalId(var_id));
                }
                FileItem::EnumStruct(id) => {
                    let enum_struct = &item_tree[*id];
                    let enum_struct_id = EnumStructLoc {
                        container: file_id.into(),
                        id: ItemTreeId {
                            tree: tree_id,
                            value: *id,
                        },
                    }
                    .intern(db);
                    res.declare(
                        enum_struct.name.clone(),
                        FileDefId::EnumStructId(enum_struct_id),
                    );
                }
            }
        }

        Arc::new(res)
    }

    pub fn get(&self, name: &Name) -> Option<FileDefId> {
        self.values.get(name).copied()
    }

    pub fn get_from_str(&self, name: &str) -> Option<FileDefId> {
        self.get(&Name::from(name))
    }

    pub(crate) fn block_def_map_query(db: &dyn DefDatabase, block_id: BlockId) -> Arc<DefMap> {
        let item_tree = db.block_item_tree(block_id);
        let file_id = block_id.lookup(db).file_id;
        let mut res = DefMap::default();
        for item in item_tree.top_level_items() {
            match item {
                FileItem::Variable(id) => {
                    let var = &item_tree[*id];
                    let var_id = GlobalLoc {
                        tree: TreeId::new(file_id, Some(block_id)),
                        value: *id,
                    }
                    .intern(db);
                    res.declare(var.name.clone(), FileDefId::GlobalId(var_id));
                }
                _ => unreachable!("Only variables can be defined in a block"),
            }
        }

        Arc::new(res)
    }

    fn declare(&mut self, name: Name, def: FileDefId) {
        self.values.insert(name, def);
        self.declarations.push(def);
    }

    pub(crate) fn block_id(&self) -> Option<BlockId> {
        self.block.map(|block| block.block)
    }

    pub fn declarations(&self) -> &[FileDefId] {
        &self.declarations
    }
}
