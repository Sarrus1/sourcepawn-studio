use std::sync::Arc;

use base_db::SourceDatabase;
use vfs::FileId;

use crate::{
    ast_id_map::AstIdMap, item_tree::ItemTree, FunctionId, FunctionLoc, VariableId, VariableLoc,
};

#[salsa::query_group(InternDatabaseStorage)]
pub trait InternDatabase: SourceDatabase {
    // region: items
    #[salsa::interned]
    fn intern_function(&'tree self, loc: FunctionLoc) -> FunctionId;
    #[salsa::interned]
    fn intern_variable(&'tree self, loc: VariableLoc) -> VariableId;
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

    #[salsa::invoke(AstIdMap::from_tree)]
    fn ast_id_map(&self, file_id: FileId) -> Arc<AstIdMap>;
}
