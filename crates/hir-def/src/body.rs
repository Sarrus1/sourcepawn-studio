use std::sync::Arc;

use syntax::TSKind;

use crate::{
    db::DefMap, src::HasSource, BlockId, BlockLoc, DefDatabase, DefWithBodyId, Intern, Lookup,
};

/// The body of a function
#[derive(Debug, Eq, PartialEq, Default)]
pub struct Body {
    block_scopes: Vec<BlockId>,
}

impl Body {
    pub(crate) fn body_query(db: &dyn DefDatabase, def: DefWithBodyId) -> Arc<Body> {
        let mut body = Body::default();
        match def {
            DefWithBodyId::FunctionId(id) => {
                let func = id.lookup(db);
                let file_id = func.file_id();
                let tree = db.parse(file_id);
                let func_node = func.source(db, &tree);
                let ast_id_map = db.ast_id_map(file_id);
                for child in func_node.value.children(&mut func_node.value.walk()) {
                    if TSKind::from(child) == TSKind::sym_block {
                        let block_id = BlockLoc {
                            ast_id: ast_id_map.ast_id_of(&child),
                            file_id,
                        }
                        .intern(db);
                        body.block_scopes.push(block_id);
                    }
                }
            }
        }
        Arc::new(body)
    }

    /// Returns an iterator over all block expressions in this body that define inner items.
    pub fn blocks<'a>(
        &'a self,
        db: &'a dyn DefDatabase,
    ) -> impl Iterator<Item = (BlockId, Arc<DefMap>)> + '_ {
        self.block_scopes
            .iter()
            .map(move |&block| (block, db.block_def_map(block)))
    }
}
