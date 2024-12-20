use crate::{
    db::DefMap,
    hir::{Expr, ExprId, Ident, IdentId},
    src::HasSource,
    BlockId, DefDatabase, DefWithBodyId, InFile, Lookup, NodePtr,
};
use fxhash::FxHashMap;
use la_arena::{Arena, ArenaMap};
use std::ops::Index;
use std::sync::Arc;
use syntax::TSKind;
use vfs::FileId;

pub mod lower;
pub mod scope;

/// The body of a function
#[derive(Debug, Eq, PartialEq, Default)]
pub struct Body {
    pub exprs: Arena<Expr>,
    pub body_expr: Option<ExprId>,
    pub idents: Arena<Ident>,
    pub params: Vec<(IdentId, ExprId)>,
    /// Block expressions in this body that may contain inner items.
    block_scopes: Vec<BlockId>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BodySourceMap {
    expr_map: FxHashMap<InFile<NodePtr>, ExprId>,
    expr_map_back: ArenaMap<ExprId, InFile<NodePtr>>,

    ident_map: FxHashMap<InFile<NodePtr>, IdentId>,
    ident_map_back: ArenaMap<IdentId, InFile<NodePtr>>,
}

impl BodySourceMap {
    pub fn expr_source(&self, expr: ExprId) -> Option<InFile<NodePtr>> {
        self.expr_map_back.get(expr).cloned()
    }

    pub fn node_expr(&self, node: InFile<&tree_sitter::Node>) -> Option<ExprId> {
        let ptr = node.map(NodePtr::from);
        self.node_ptr_expr(ptr)
    }

    pub fn node_ptr_expr(&self, node: InFile<NodePtr>) -> Option<ExprId> {
        self.expr_map.get(&node).cloned()
    }

    pub fn node_ident(&self, node: &InFile<NodePtr>) -> Option<IdentId> {
        self.ident_map.get(node).cloned()
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
                let file_id = func.id.file_id();
                let tree = db.parse(file_id);
                let InFile {
                    file_id,
                    value: func_node,
                } = func.source(db, &tree);
                let body_node = func_node.child_by_field_name("body");
                let params_list = match TSKind::from(func_node) {
                    TSKind::methodmap_property_native | TSKind::methodmap_property_method => {
                        if let Some(setter_node) = func_node
                            .children(&mut func_node.walk())
                            .find(|child| TSKind::from(child) == TSKind::methodmap_property_setter)
                        {
                            setter_node.child_by_field_name("parameter")
                        } else {
                            None
                        }
                    }
                    _ => func_node
                        .children(&mut func_node.walk())
                        .find(|child| TSKind::from(child) == TSKind::parameter_declarations),
                };
                let (body, sourcemap) = Body::new(
                    db,
                    def,
                    file_id,
                    &db.preprocessed_text(file_id),
                    params_list,
                    body_node,
                );
                (Arc::new(body), Arc::new(sourcemap))
            }
            DefWithBodyId::TypedefId(id) => {
                let typedef = id.lookup(db);
                let file_id = typedef.id.file_id();
                let tree = db.parse(file_id);
                let InFile {
                    file_id,
                    value: typedef_node,
                } = typedef.source(db, &tree);
                let params_list = match TSKind::from(typedef_node) {
                    TSKind::typedef => typedef_node
                        .children(&mut typedef_node.walk())
                        .find(|child| TSKind::from(child) == TSKind::typedef_expression)
                        .and_then(|n| n.child_by_field_name("parameters")),
                    TSKind::typedef_expression => typedef_node.child_by_field_name("parameters"),
                    _ => None,
                };
                let (body, sourcemap) = Body::new(
                    db,
                    def,
                    file_id,
                    &db.preprocessed_text(file_id),
                    params_list,
                    None,
                );
                (Arc::new(body), Arc::new(sourcemap))
            }
            DefWithBodyId::FunctagId(id) => {
                let functag = id.lookup(db);
                let file_id = functag.id.file_id();
                let tree = db.parse(file_id);
                let InFile {
                    file_id,
                    value: functag_node,
                } = functag.source(db, &tree);
                let params_list = functag_node.child_by_field_name("parameters");
                let (body, sourcemap) = Body::new(
                    db,
                    def,
                    file_id,
                    &db.preprocessed_text(file_id),
                    params_list,
                    None,
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
    ) -> impl Iterator<Item = (BlockId, Arc<DefMap>)> + 'a {
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
