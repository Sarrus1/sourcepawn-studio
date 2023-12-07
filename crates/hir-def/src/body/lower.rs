use std::sync::Arc;

use syntax::TSKind;
use vfs::FileId;

use crate::{
    ast_id_map::AstIdMap,
    db::DefMap,
    hir::{Expr, ExprId, Ident},
    item_tree::Name,
    BlockLoc, DefDatabase, DefWithBodyId, InFile, NodePtr,
};

use super::{Body, BodySourceMap};

pub(super) fn lower(
    db: &dyn DefDatabase,
    owner: DefWithBodyId,
    // params: Option<(ast::ParamList, impl Iterator<Item = bool>)>,
    file_id: FileId,
    source: &str,
    body: Option<tree_sitter::Node>,
) -> (Body, BodySourceMap) {
    ExprCollector {
        db,
        file_id,
        source,
        owner,
        // def_map: expander.module.def_map(db),
        source_map: BodySourceMap::default(),
        ast_id_map: db.ast_id_map(file_id),
        body: Body::default(),
    }
    .collect(
        // params,
        body,
    )
}

struct ExprCollector<'a> {
    db: &'a dyn DefDatabase,
    file_id: FileId,
    source: &'a str,
    owner: DefWithBodyId,
    // def_map: Arc<DefMap>,
    ast_id_map: Arc<AstIdMap>,
    body: Body,
    source_map: BodySourceMap,
}

impl ExprCollector<'_> {
    fn collect(
        mut self,
        // param_list: Option<(ast::ParamList, impl Iterator<Item = bool>)>,
        body: Option<tree_sitter::Node>,
    ) -> (Body, BodySourceMap) {
        if let Some(body) = body {
            self.body.body_expr = self.collect_expr(body);
        }
        (self.body, self.source_map)
    }

    fn collect_variable_declaration(&mut self, expr: tree_sitter::Node) -> ExprId {
        let mut decl = vec![];
        for child in expr.children(&mut expr.walk()) {
            if TSKind::from(child) == TSKind::sym_variable_declaration {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let ident_id = self
                        .body
                        .idents
                        .alloc(Name::from_node(&name_node, self.source));
                    let binding_id = self.alloc_expr(Expr::Binding, NodePtr::from(&child));
                    decl.push((ident_id, binding_id, None));
                }
            }
        }
        let decl = Expr::Decl(decl);
        self.alloc_expr(decl, NodePtr::from(&expr))
    }

    fn collect_expr(&mut self, expr: tree_sitter::Node) -> ExprId {
        match TSKind::from(expr) {
            TSKind::sym_block => {
                let ast_id = self.ast_id_map.ast_id_of(&expr);
                let block_id = self.db.intern_block(BlockLoc {
                    ast_id,
                    file_id: self.file_id,
                });
                let mut statements = Vec::new();
                for child in expr.children(&mut expr.walk()) {
                    match TSKind::from(child) {
                        TSKind::anon_sym_LBRACE | TSKind::anon_sym_RBRACE => continue,
                        _ => (),
                    }
                    statements.push(self.collect_expr(child));
                }
                let block = Expr::Block {
                    id: Some(block_id),
                    statements: statements.into_boxed_slice(),
                };
                self.alloc_expr(block, NodePtr::from(&expr))
            }
            TSKind::sym_expression_statement => {
                let mut cursor = expr.walk();
                let child = expr.children(&mut cursor).next().unwrap(); // FIXME: This is bad, use Options
                self.collect_expr(child)
            }
            TSKind::sym_variable_declaration_statement => self.collect_variable_declaration(expr),
            _ => todo!(
                "Expression collector for {:?} is not implemented",
                TSKind::from(expr)
            ),
        }
    }

    fn alloc_expr(&mut self, expr: Expr, ptr: NodePtr) -> ExprId {
        let id = self.body.exprs.alloc(expr);
        self.source_map.expr_map_back.insert(id, ptr);
        self.source_map.expr_map.insert(ptr, id);
        id
    }
}
