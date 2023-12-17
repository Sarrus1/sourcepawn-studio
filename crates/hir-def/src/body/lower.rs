use std::sync::Arc;

use syntax::TSKind;
use vfs::FileId;

use crate::{
    ast_id_map::AstIdMap,
    hir::{Expr, ExprId},
    item_tree::Name,
    BlockLoc, DefDatabase, DefWithBodyId, NodePtr,
};

use super::{Body, BodySourceMap};

pub(super) fn lower(
    db: &dyn DefDatabase,
    owner: DefWithBodyId,
    params_list: Option<tree_sitter::Node>,
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
    .collect(params_list, body)
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
        params_list: Option<tree_sitter::Node>,
        body: Option<tree_sitter::Node>,
    ) -> (Body, BodySourceMap) {
        if let Some(params_list) = params_list {
            match TSKind::from(params_list) {
                TSKind::sym_parameter_declarations => {
                    for child in params_list.children(&mut params_list.walk()) {
                        if TSKind::from(child) == TSKind::sym_parameter_declaration {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                let ident_id = self
                                    .body
                                    .idents
                                    .alloc(Name::from_node(&name_node, self.source));
                                let binding_id =
                                    self.alloc_expr(Expr::Binding, NodePtr::from(&child));
                                self.body.params.push((ident_id, binding_id));
                            }
                        }
                    }
                }
                _ => todo!("Handle non argument declarations"),
            }
        }
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
        self.maybe_collect_expr(expr)
            .unwrap_or_else(|| self.missing_expr())
    }

    fn maybe_collect_expr(&mut self, expr: tree_sitter::Node) -> Option<ExprId> {
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
                Some(self.alloc_expr(block, NodePtr::from(&expr)))
            }
            TSKind::sym_expression_statement => {
                let child = expr.children(&mut expr.walk()).next()?;
                Some(self.collect_expr(child))
            }
            TSKind::sym_field_access => {
                let field_access = Expr::FieldAccess {
                    target: self.collect_expr(expr.child_by_field_name("target")?),
                    field: Name::from_node(&expr.child_by_field_name("field")?, self.source),
                };
                Some(self.alloc_expr(field_access, NodePtr::from(&expr)))
            }
            TSKind::sym_variable_declaration_statement => {
                Some(self.collect_variable_declaration(expr))
            }
            _ => {
                log::warn!("Unhandled expression: {:?}", expr);
                None
            }
        }
    }

    fn alloc_expr_desugared(&mut self, expr: Expr) -> ExprId {
        self.body.exprs.alloc(expr)
    }

    fn missing_expr(&mut self) -> ExprId {
        self.alloc_expr_desugared(Expr::Missing)
    }

    fn alloc_expr(&mut self, expr: Expr, ptr: NodePtr) -> ExprId {
        let id = self.body.exprs.alloc(expr);
        self.source_map.expr_map_back.insert(id, ptr);
        self.source_map.expr_map.insert(ptr, id);
        id
    }
}
