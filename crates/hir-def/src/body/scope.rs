use std::sync::Arc;

use fxhash::FxHashMap;
use la_arena::{Arena, ArenaMap, Idx};
use vfs::FileId;

use crate::{
    hir::{Expr, ExprId},
    item_tree::Name,
    DefDatabase, DefWithBodyId,
};

use super::Body;

pub type ScopeId = Idx<ScopeData>;

#[derive(Debug, PartialEq, Eq)]
pub enum ScopeParent {
    Block(ScopeId),
    File(FileId),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExprScopes {
    scopes: Arena<ScopeData>,
    scope_entries: Arena<ExprId>,
    scope_by_expr: ArenaMap<ExprId, ScopeId>,
}

impl ExprScopes {
    pub fn expr_scopes_query(
        db: &dyn DefDatabase,
        def: DefWithBodyId,
        file_id: FileId,
    ) -> Arc<Self> {
        let body = db.body(def);
        let mut scopes = ExprScopes::new(&body, file_id);
        scopes.shrink_to_fit();
        Arc::new(scopes)
    }

    /// If `scope` refers to a file scope, returns the corresponding `FileId`.
    pub fn file_id(&self, scope: ScopeId) -> Option<FileId> {
        match self.scopes[scope].parent {
            ScopeParent::File(file_id) => Some(file_id),
            _ => None,
        }
    }

    /// Returns the scopes in ascending order.
    pub fn scope_chain(&self, scope: Option<ScopeId>) -> impl Iterator<Item = ScopeId> + '_ {
        std::iter::successors(scope, move |&scope| self.scopes[scope].scope_parent())
    }

    pub fn resolve_name_in_scope(&self, scope: ScopeId, name: &Name) -> Option<&ScopeEntry> {
        self.scope_chain(Some(scope))
            .find_map(|scope| self.scopes[scope].entries.get(name))
    }

    pub fn scope_for(&self, expr: ExprId) -> Option<ScopeId> {
        self.scope_by_expr.get(expr).copied()
    }

    pub fn scope_by_expr(&self) -> &ArenaMap<ExprId, ScopeId> {
        &self.scope_by_expr
    }

    pub fn entry(&self, entry: ScopeEntry) -> &ExprId {
        &self.scope_entries[entry]
    }

    pub fn first_scope(&self) -> Option<ScopeId> {
        self.scopes.iter().next().map(|(id, _)| id)
    }

    pub fn entries(&self, scope: ScopeId) -> impl Iterator<Item = (&Name, &ScopeEntry)> + '_ {
        self.scope_chain(Some(scope))
            .flat_map(move |scope| self.scopes[scope].entries.iter())
    }
}

impl ExprScopes {
    fn new(body: &Body, file_id: FileId) -> Self {
        let mut scopes = ExprScopes {
            scopes: Arena::default(),
            scope_entries: Arena::default(),
            scope_by_expr: ArenaMap::with_capacity(body.exprs.len()),
        };
        let mut root = scopes.root_scope(file_id);
        scopes.add_params_bindings(body, root);
        if let Some(id) = body.body_expr {
            compute_expr_scopes(id, body, &mut scopes, &mut root);
        }
        scopes
    }

    fn add_params_bindings(&mut self, body: &Body, scope: ScopeId) {
        for (ident_id, binding_id) in body.params.iter() {
            let binding = self.scope_entries.alloc(*binding_id);
            self.scopes[scope]
                .entries
                .insert(body.idents[*ident_id].clone(), binding);
        }
    }

    fn root_scope(&mut self, file_id: FileId) -> ScopeId {
        self.scopes.alloc(ScopeData {
            parent: ScopeParent::File(file_id),
            entries: FxHashMap::default(),
        })
    }

    fn new_block_scope(&mut self, parent: ScopeId) -> ScopeId {
        self.scopes.alloc(ScopeData {
            parent: ScopeParent::Block(parent),
            entries: FxHashMap::default(),
        })
    }

    /// Sets a scope mapping for the given block expression.
    fn set_scope(&mut self, node: ExprId, scope: ScopeId) {
        self.scope_by_expr.insert(node, scope);
    }

    fn shrink_to_fit(&mut self) {
        let ExprScopes {
            scopes,
            scope_entries,
            scope_by_expr,
        } = self;
        scopes.shrink_to_fit();
        scope_entries.shrink_to_fit();
        scope_by_expr.shrink_to_fit();
    }
}

pub type ScopeEntry = Idx<ExprId>;

#[derive(Debug, PartialEq, Eq)]
pub struct ScopeData {
    parent: ScopeParent,
    entries: FxHashMap<Name, ScopeEntry>,
}

impl ScopeData {
    pub fn scope_parent(&self) -> Option<ScopeId> {
        match self.parent {
            ScopeParent::Block(block) => Some(block),
            ScopeParent::File(_) => None,
        }
    }
}

/// Compute the [`ExprScopes`](ExprScopes) from the [`exprs`](Expr) of the [`body`](Body), by populating each scope
/// with the declarations that were made in that scope.
///
/// For SourcePawn, this is only variable declarations, as the language does not have closures, etc (yet).
fn compute_expr_scopes(expr: ExprId, body: &Body, scopes: &mut ExprScopes, scope: &mut ScopeId) {
    scopes.set_scope(expr, *scope);
    match &body[expr] {
        Expr::Decl(decl) => {
            for binding in decl.iter() {
                compute_expr_scopes(*binding, body, scopes, scope);
            }
        }
        Expr::Binding { ident_id, .. } => {
            let binding = scopes.scope_entries.alloc(expr);
            scopes.scopes[*scope]
                .entries
                .insert(body.idents[*ident_id].clone(), binding);
        }
        Expr::Block { statements, .. } => {
            let mut scope = scopes.new_block_scope(*scope);
            // Overwrite the old scope for the block expr, so that every block scope can be found
            // via the block itself (important for blocks that only contain items, no expressions).
            scopes.set_scope(expr, scope);
            for &stmt in statements.iter() {
                compute_expr_scopes(stmt, body, scopes, &mut scope);
            }
        }
        Expr::Loop {
            initialization,
            body: loop_body,
            ..
        } => {
            for init in initialization.iter() {
                compute_expr_scopes(*init, body, scopes, scope);
            }
            if let Some(loop_body) = loop_body {
                compute_expr_scopes(*loop_body, body, scopes, scope);
            }
        }
        Expr::Condition {
            then_branch,
            else_branch,
            ..
        } => {
            compute_expr_scopes(*then_branch, body, scopes, scope);
            if let Some(else_branch) = else_branch {
                compute_expr_scopes(*else_branch, body, scopes, scope);
            }
        }
        Expr::Switch { cases, .. } => {
            for case in cases.iter() {
                compute_expr_scopes(case.body(), body, scopes, scope);
            }
        }
        // These expressions do not introduce any declarations.
        Expr::Missing
        | Expr::Control { .. }
        | Expr::NamedArg { .. }
        | Expr::CommaExpr { .. }
        | Expr::ScopeAccess { .. }
        | Expr::ArrayIndexedAccess { .. }
        | Expr::Ident(_)
        | Expr::This
        | Expr::New { .. }
        | Expr::ViewAs { .. }
        | Expr::FieldAccess { .. }
        | Expr::BinaryOp { .. }
        | Expr::UnaryOp { .. }
        | Expr::TernaryOp { .. }
        | Expr::Call { .. }
        | Expr::MethodCall { .. }
        | Expr::DynamicArray { .. }
        | Expr::Literal(_) => (),
    };
}
