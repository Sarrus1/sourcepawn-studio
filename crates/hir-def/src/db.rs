use std::sync::Arc;

use base_db::SourceDatabase;
use fxhash::FxHashMap;
use vfs::FileId;

use crate::{
    ast_id_map::AstIdMap,
    body::{scope::ExprScopes, Body, BodySourceMap},
    item_tree::{ItemTree, Name},
    BlockId, BlockLoc, DefWithBodyId, FileDefId, FileItem, FunctionId, FunctionLoc, Intern, Lookup,
    TreeId, VariableId, VariableLoc,
};

#[salsa::query_group(InternDatabaseStorage)]
pub trait InternDatabase: SourceDatabase {
    // region: items
    #[salsa::interned]
    fn intern_function(&'tree self, loc: FunctionLoc) -> FunctionId;
    #[salsa::interned]
    fn intern_variable(&'tree self, loc: VariableLoc) -> VariableId;
    #[salsa::interned]
    fn intern_block(&'tree self, loc: BlockLoc) -> BlockId;
    // endregion: items

    /*
    #[salsa::interned]
    fn intern_block(&self, loc: BlockLoc) -> BlockId;
    #[salsa::interned]
    fn intern_anonymous_const(&self, id: ConstBlockLoc) -> ConstBlockId;
    #[salsa::interned]
    fn intern_in_type_const(&self, id: InTypeConstLoc) -> InTypeConstId;
    */
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
}

/// For `DefMap`s computed for a block expression, this stores its location in the parent map.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct BlockInfo {
    /// The `BlockId` this `DefMap` was created from.
    block: BlockId,
    /// The containing file.
    parent: FileId,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct DefMap {
    values: FxHashMap<Name, FileDefId>,
    /// When this is a block def map, this will hold the block id of the block and module that
    /// contains this block.
    block: Option<BlockInfo>,
}

impl DefMap {
    pub fn file_def_map_query(db: &dyn DefDatabase, file_id: FileId) -> Arc<Self> {
        let item_tree = db.file_item_tree(file_id);
        let mut res = DefMap::default();
        for item in item_tree.top_level_items() {
            match item {
                FileItem::Function(id) => {
                    let func = &item_tree[*id];
                    let fn_id = FunctionLoc {
                        tree: TreeId::new(file_id, None), // TODO: Reuse the file_id with "into" ?
                        value: *id,
                    }
                    .intern(db);

                    res.values
                        .insert(func.name.clone(), FileDefId::FunctionId(fn_id));
                }
                FileItem::Variable(id) => {
                    let var = &item_tree[*id];
                    let var_id = VariableLoc {
                        tree: TreeId::new(file_id, None), // TODO: Reuse the file_id with "into" ?
                        value: *id,
                    }
                    .intern(db);
                    res.values
                        .insert(var.name.clone(), FileDefId::VariableId(var_id));
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
                    let var_id = VariableLoc {
                        tree: TreeId::new(file_id, Some(block_id)),
                        value: *id,
                    }
                    .intern(db);
                    res.values
                        .insert(var.name.clone(), FileDefId::VariableId(var_id));
                }
                _ => unreachable!("Only variables can be defined in a block"),
            }
        }

        Arc::new(res)
    }

    pub(crate) fn block_id(&self) -> Option<BlockId> {
        self.block.map(|block| block.block)
    }
}
