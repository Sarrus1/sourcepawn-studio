use std::{fmt, sync::Arc};

use crate::{
    body::scope::{ExprScopes, ScopeData, ScopeEntry, ScopeId},
    db::DefMap,
    hir::ExprId,
    item_tree::Name,
    DefDatabase, DefWithBodyId, FileDefId, FunctionId, InFile, Lookup, NodePtr, TreeId, VariableId,
};
use la_arena::{Arena, ArenaMap, Idx};
use vfs::FileId;

#[derive(Debug, Clone)]
pub struct Resolver {
    /// The stack of scopes, where the inner-most scope is the last item.
    ///
    /// When using, you generally want to process the scopes in reverse order,
    /// there's `scopes` *method* for that.
    scopes: Vec<Scope>,
    file_id: FileId,
    // module_scope: Arc<DefMap>,
}

#[derive(Clone)]
struct ExprScope {
    owner: DefWithBodyId,
    expr_scopes: Arc<ExprScopes>,
    scope_id: ScopeId,
}

impl ExprScope {
    pub fn resolve_name_in_scope(&self, name: &Name) -> Option<ExprId> {
        self.expr_scopes
            .resolve_name_in_scope(self.scope_id, name)
            .cloned()
            .map(|entry| self.expr_scopes.entry(entry))
            .cloned()
    }
}

impl fmt::Debug for ExprScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExprScope")
            .field("owner", &self.owner)
            .field("scope_id", &self.scope_id)
            .finish()
    }
}

#[derive(Debug, Clone)]
enum Scope {
    /// All the items and included names of a project.
    GlobalScope(Arc<DefMap>),
    /// Brings `this` into scope.
    // ThisScope(ImplId),
    /// Local bindings.
    ExprScope(ExprScope),
}

impl Resolver {
    fn scopes(&self) -> impl Iterator<Item = &Scope> {
        self.scopes.iter().rev()
    }

    fn push_scope(mut self, scope: Scope) -> Resolver {
        self.scopes.push(scope);
        self
    }

    fn push_global_scope(self, def_map: Arc<DefMap>, file_id: FileId) -> Self {
        self.push_scope(Scope::GlobalScope(def_map))
    }

    fn push_expr_scope(
        self,
        owner: DefWithBodyId,
        expr_scopes: Arc<ExprScopes>,
        scope_id: ScopeId,
    ) -> Resolver {
        self.push_scope(Scope::ExprScope(ExprScope {
            owner,
            expr_scopes,
            scope_id,
        }))
    }
}

impl Resolver {
    pub fn resolve_ident(&self, name: &str) -> Option<ValueNs> {
        let name = Name::from(name);
        for scope in self.scopes() {
            match scope {
                Scope::ExprScope(scope) => {
                    if let Some(entry) = scope.resolve_name_in_scope(&name) {
                        return Some(ValueNs::LocalVariable(entry));
                    }
                }
                _ => todo!(),
            }
        }
        None
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ValueNs {
    LocalVariable(ExprId),
    GlobalVariable(InFile<VariableId>),
    FunctionId(InFile<FunctionId>),
}

pub trait HasResolver: Copy {
    /// Builds a resolver for type references inside this def.
    fn resolver(self, db: &dyn DefDatabase) -> Resolver;
}

impl HasResolver for DefWithBodyId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        match self {
            Self::FunctionId(it) => it.resolver(db),
        }
    }
}

impl HasResolver for FunctionId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        self.lookup(db).file_id().resolver(db)
        // .push_generic_params_scope(db, self.into())
    }
}

impl HasResolver for FileId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        Resolver {
            scopes: vec![Scope::GlobalScope(db.file_def_map(self))],
            file_id: self,
        }
    }
}

pub fn resolver_for_scope(
    db: &dyn DefDatabase,
    owner: DefWithBodyId,
    scope_id: Option<ScopeId>,
) -> Resolver {
    let mut resolver = owner.resolver(db);
    let scopes = db.expr_scopes(owner, resolver.file_id);
    let scope_chain = scopes.scope_chain(scope_id).collect::<Vec<_>>();
    resolver.scopes.reserve(scope_chain.len());

    for scope in scope_chain.into_iter().rev() {
        match scopes.file_id(scope) {
            Some(file_id) => {
                let def_map = db.file_def_map(file_id);
                resolver = resolver.push_global_scope(def_map, file_id);
            }
            None => resolver = resolver.push_expr_scope(owner, Arc::clone(&scopes), scope),
        }
    }
    resolver
}
