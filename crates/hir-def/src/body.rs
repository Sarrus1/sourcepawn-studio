use crate::{
    db::DefMap,
    hir::{Expr, ExprId, Ident, IdentId},
    src::HasSource,
    BlockId, BlockLoc, DefDatabase, DefWithBodyId, InFile, Intern, Lookup, NodePtr,
};
use fxhash::FxHashMap;
use la_arena::{Arena, ArenaMap, RawIdx};
use std::ops::Index;
use std::sync::Arc;
use syntax::TSKind;
use vfs::FileId;

pub mod lower;
pub mod scope;

/// The body of a function
#[derive(Debug, Eq, PartialEq)]
pub struct Body {
    pub exprs: Arena<Expr>,
    pub body_expr: ExprId,
    pub idents: Arena<Ident>,
    pub params: Vec<(IdentId, ExprId)>,
    /// Block expressions in this body that may contain inner items.
    block_scopes: Vec<BlockId>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BodySourceMap {
    expr_map: FxHashMap<NodePtr, ExprId>,
    expr_map_back: ArenaMap<ExprId, NodePtr>,

    ident_map: FxHashMap<NodePtr, IdentId>,
    ident_map_back: ArenaMap<IdentId, NodePtr>,
}

impl BodySourceMap {
    pub fn expr_source(&self, expr: ExprId) -> Option<NodePtr> {
        self.expr_map_back.get(expr).cloned()
    }

    pub fn node_expr(&self, node: &tree_sitter::Node) -> Option<ExprId> {
        let ptr = NodePtr::from(node);
        self.expr_map.get(&ptr).cloned()
    }
}

impl Body {
    pub(crate) fn body_with_source_map_query(
        db: &dyn DefDatabase,
        def: DefWithBodyId,
    ) -> (Arc<Body>, Arc<BodySourceMap>) {
        match def {
            DefWithBodyId::FunctionId(id) => {
                let func = id.lookup(db);
                let file_id = func.file_id();
                let tree = db.parse(file_id);
                let InFile {
                    file_id,
                    value: func_node,
                } = func.source(db, &tree);
                let body_node = func_node.child_by_field_name("body");
                let params_list = func_node
                    .children(&mut func_node.walk())
                    .find(|child| TSKind::from(child) == TSKind::sym_argument_declarations);
                let (body, sourcemap) = Body::new(
                    db,
                    def,
                    file_id,
                    &db.file_text(file_id),
                    params_list,
                    body_node,
                );
                (Arc::new(body), Arc::new(sourcemap))
            }
        }
    }

    pub(crate) fn body_query(db: &dyn DefDatabase, def: DefWithBodyId) -> Arc<Body> {
        let (body, _) = db.body_with_source_map(def);
        body
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

    fn new(
        db: &dyn DefDatabase,
        owner: DefWithBodyId,
        file_id: FileId,
        source: &str,
        // expander: Expander,
        params_list: Option<tree_sitter::Node>,
        body: Option<tree_sitter::Node>,
    ) -> (Body, BodySourceMap) {
        lower::lower(db, owner, params_list, file_id, source, body)
    }
}

impl Default for Body {
    fn default() -> Self {
        Self {
            body_expr: ExprId::from_raw(RawIdx::from(u32::MAX)), // HACK: Initialize with invalid index
            exprs: Default::default(),
            idents: Default::default(),
            params: Default::default(),
            block_scopes: Default::default(),
        }
    }
}

impl Index<ExprId> for Body {
    type Output = Expr;

    fn index(&self, expr: ExprId) -> &Expr {
        &self.exprs[expr]
    }
}

impl Index<IdentId> for Body {
    type Output = Ident;

    fn index(&self, ident: IdentId) -> &Ident {
        &self.idents[ident]
    }
}
