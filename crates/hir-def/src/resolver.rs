use std::{fmt, sync::Arc};

use crate::{
    body::scope::{ExprScopes, ScopeId},
    db::DefMap,
    hir::ExprId,
    item_tree::Name,
    AdtId, DefDatabase, DefWithBodyId, EnumId, EnumStructId, FileDefId, FuncenumId, FunctagId,
    FunctionId, GlobalId, InFile, ItemContainerId, Lookup, MacroId, MethodmapId, PropertyId,
    TypedefId, TypesetId, VariantId,
};
use itertools::Itertools;
use smallvec::SmallVec;
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

    pub fn entries(&self) -> impl Iterator<Item = (&Name, &ExprId)> + '_ {
        self.expr_scopes
            .entries(self.scope_id)
            .map(|(name, entry)| (name, self.expr_scopes.entry(*entry)))
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
    Global(Vec<Arc<DefMap>>),
    /// Brings `this` into scope.
    This(AdtId),
    /// Local bindings.
    Expr(ExprScope),
}

impl Resolver {
    fn scopes(&self) -> impl Iterator<Item = &Scope> {
        self.scopes.iter().rev()
    }

    fn push_scope(mut self, scope: Scope) -> Resolver {
        self.scopes.push(scope);
        self
    }

    fn push_this_scope(self, adt_id: AdtId) -> Resolver {
        self.push_scope(Scope::This(adt_id))
    }

    fn push_expr_scope(
        self,
        owner: DefWithBodyId,
        expr_scopes: Arc<ExprScopes>,
        scope_id: ScopeId,
    ) -> Resolver {
        self.push_scope(Scope::Expr(ExprScope {
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
                Scope::Expr(scope) => {
                    if let Some(entry) = scope.resolve_name_in_scope(&name) {
                        return Some(ValueNs::LocalId((name.into(), scope.owner, entry)));
                    }
                }
                Scope::This(adt_id) => {
                    if name != "this".into() {
                        continue;
                    }
                    match adt_id {
                        AdtId::EnumStructId(it) => {
                            return Some(ValueNs::EnumStructId(InFile::new(self.file_id, *it)));
                        }
                        AdtId::MethodmapId(it) => {
                            return Some(ValueNs::MethodmapId(InFile::new(self.file_id, *it)));
                        }
                    }
                }
                Scope::Global(def_maps) => {
                    let mut entries: Vec<(FileDefId, FileId)> = vec![];
                    def_maps.iter().for_each(|def_map| {
                        if let Some(entry) = def_map.get(&name) {
                            entries
                                .extend(entry.into_iter().map(|entry| (entry, def_map.file_id())));
                        }
                    });
                    entries.dedup(); // FIXME: Use a HashSet instead of Vec
                    match entries.len() {
                        0 => continue,
                        1 => return to_valuens(entries[0].0, entries[0].1),
                        _ => {
                            // Handle enum methodmaps by returning the methodmap id in priority if it exists
                            if let Some((FileDefId::MethodmapId(it), file_id)) = entries
                                .iter()
                                .find(|(entry, _)| matches!(entry, FileDefId::MethodmapId(_)))
                            {
                                return Some(ValueNs::MethodmapId(InFile::new(*file_id, *it)));
                            }

                            let mut fn_ids: SmallVec<[InFile<FunctionId>; 1]> = SmallVec::new();
                            for entry in entries {
                                if let (FileDefId::FunctionId(it), file_id) = entry {
                                    fn_ids.push(InFile::new(file_id, it));
                                }
                            }
                            return Some(ValueNs::FunctionId(fn_ids));
                        }
                    }
                }
            }
        }
        None
    }

    /// `expr_id` is required to be an expression id that comes after the top level expression scope in the given resolver
    #[must_use]
    pub fn update_to_inner_scope(
        &mut self,
        db: &dyn DefDatabase,
        owner: DefWithBodyId,
        expr_id: ExprId,
    ) -> UpdateGuard {
        #[inline(always)]
        fn append_expr_scope(
            _db: &dyn DefDatabase,
            resolver: &mut Resolver,
            owner: DefWithBodyId,
            expr_scopes: &Arc<ExprScopes>,
            scope_id: ScopeId,
        ) {
            resolver.scopes.push(Scope::Expr(ExprScope {
                owner,
                expr_scopes: expr_scopes.clone(),
                scope_id,
            }));
        }

        let start = self.scopes.len();
        let innermost_scope = self.scopes().next();
        match innermost_scope {
            Some(&Scope::Expr(ExprScope {
                scope_id,
                ref expr_scopes,
                owner,
            })) => {
                let expr_scopes = expr_scopes.clone();
                let scope_chain = expr_scopes
                    .scope_chain(expr_scopes.scope_for(expr_id))
                    .take_while(|&it| it != scope_id);
                for scope_id in scope_chain {
                    append_expr_scope(db, self, owner, &expr_scopes, scope_id);
                }
            }
            _ => {
                let expr_scopes = db.expr_scopes(owner, self.file_id);
                let scope_chain = expr_scopes.scope_chain(expr_scopes.scope_for(expr_id));

                for scope_id in scope_chain {
                    append_expr_scope(db, self, owner, &expr_scopes, scope_id);
                }
            }
        }
        self.scopes[start..].reverse();
        UpdateGuard(start)
    }

    pub fn reset_to_guard(&mut self, UpdateGuard(start): UpdateGuard) {
        self.scopes.truncate(start);
    }

    /// Update the resolver to the outer most local scope of the owner.
    pub fn update_to_first_local_scope(&mut self, db: &dyn DefDatabase, owner: DefWithBodyId) {
        let expr_scopes = db.expr_scopes(owner, self.file_id);
        let scope_id = expr_scopes.first_scope();
        if let Some(scope_id) = scope_id {
            self.scopes.push(Scope::Expr(ExprScope {
                owner,
                expr_scopes: Arc::clone(&expr_scopes),
                scope_id,
            }));
        }
    }

    pub fn available_defs(&self) -> Vec<ValueNs> {
        self.scopes()
            .flat_map(|scope| match scope {
                Scope::Global(def_maps) => def_maps
                    .iter()
                    .flat_map(|def_map| {
                        def_map
                            .declarations()
                            .iter()
                            .flat_map(|it| to_valuens(*it, def_map.file_id()))
                    })
                    .collect_vec(),
                Scope::Expr(it) => it
                    .entries()
                    .map(|(name, entry)| ValueNs::LocalId((name.clone().into(), it.owner, *entry)))
                    .collect_vec(),
                Scope::This(_) => Vec::new(),
            })
            .collect()
    }
}

fn to_valuens(entry: FileDefId, file_id: FileId) -> Option<ValueNs> {
    match (entry, file_id) {
        (FileDefId::FunctionId(it), file_id) => {
            let mut fn_ids: SmallVec<[InFile<FunctionId>; 1]> = SmallVec::new();
            fn_ids.push(InFile::new(file_id, it));
            ValueNs::FunctionId(fn_ids)
        }
        (FileDefId::MacroId(it), file_id) => ValueNs::MacroId(InFile::new(file_id, it)),
        (FileDefId::GlobalId(it), file_id) => ValueNs::GlobalId(InFile::new(file_id, it)),
        (FileDefId::EnumStructId(it), file_id) => ValueNs::EnumStructId(InFile::new(file_id, it)),
        (FileDefId::MethodmapId(it), file_id) => ValueNs::MethodmapId(InFile::new(file_id, it)),
        (FileDefId::EnumId(it), file_id) => ValueNs::EnumId(InFile::new(file_id, it)),
        (FileDefId::VariantId(it), file_id) => ValueNs::VariantId(InFile::new(file_id, it)),
        (FileDefId::TypedefId(it), file_id) => ValueNs::TypedefId(InFile::new(file_id, it)),
        (FileDefId::TypesetId(it), file_id) => ValueNs::TypesetId(InFile::new(file_id, it)),
        (FileDefId::FunctagId(it), file_id) => ValueNs::FunctagId(InFile::new(file_id, it)),
        (FileDefId::FuncenumId(it), file_id) => ValueNs::FuncenumId(InFile::new(file_id, it)),
    }
    .into()
}

pub struct UpdateGuard(usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueNs {
    LocalId((Option<Name>, DefWithBodyId, ExprId)),
    GlobalId(InFile<GlobalId>),
    MacroId(InFile<MacroId>),
    FunctionId(SmallVec<[InFile<FunctionId>; 1]>),
    EnumStructId(InFile<EnumStructId>),
    MethodmapId(InFile<MethodmapId>),
    EnumId(InFile<EnumId>),
    VariantId(InFile<VariantId>),
    TypedefId(InFile<TypedefId>),
    TypesetId(InFile<TypesetId>),
    FunctagId(InFile<FunctagId>),
    FuncenumId(InFile<FuncenumId>),
}

pub trait HasResolver: Copy {
    /// Builds a resolver for type references inside this def.
    fn resolver(self, db: &dyn DefDatabase) -> Resolver;
}

impl HasResolver for ItemContainerId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        match self {
            ItemContainerId::FileId(file_id) => file_id.resolver(db),
            ItemContainerId::EnumStructId(it) => it.resolver(db),
            ItemContainerId::MethodmapId(it) => it.resolver(db),
            ItemContainerId::TypesetId(it) => it.resolver(db),
            ItemContainerId::FuncenumId(it) => it.resolver(db),
            ItemContainerId::EnumId(it) => it.lookup(db).id.file_id().resolver(db),
        }
    }
}

impl HasResolver for TypesetId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        self.lookup(db).id.file_id().resolver(db)
    }
}

impl HasResolver for FuncenumId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        self.lookup(db).id.file_id().resolver(db)
    }
}

impl HasResolver for EnumStructId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        self.lookup(db)
            .id
            .file_id()
            .resolver(db)
            .push_this_scope(self.into())
    }
}

impl HasResolver for MethodmapId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        self.lookup(db)
            .id
            .file_id()
            .resolver(db)
            .push_this_scope(self.into())
    }
}

impl HasResolver for DefWithBodyId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        match self {
            Self::FunctionId(it) => it.resolver(db),
            Self::TypedefId(it) => it.resolver(db),
            Self::FunctagId(it) => it.resolver(db),
        }
    }
}

impl HasResolver for FunctionId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        self.lookup(db).container.resolver(db)
    }
}

impl HasResolver for PropertyId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        self.lookup(db).container.resolver(db)
    }
}

impl HasResolver for GlobalId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        self.lookup(db).file_id().resolver(db)
    }
}

impl HasResolver for TypedefId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        self.lookup(db).container.resolver(db)
    }
}

impl HasResolver for FunctagId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        self.lookup(db).container.resolver(db)
    }
}

impl HasResolver for FileId {
    fn resolver(self, db: &dyn DefDatabase) -> Resolver {
        Resolver {
            scopes: vec![Scope::Global(file_def_maps(db, self))],
            file_id: self,
        }
    }
}

pub fn global_resolver(db: &dyn DefDatabase, file_id: FileId) -> Resolver {
    file_id.resolver(db)
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
        if scopes.file_id(scope).is_none() {
            resolver = resolver.push_expr_scope(owner, Arc::clone(&scopes), scope);
        }
    }
    resolver
}

fn file_def_maps(db: &dyn DefDatabase, file_id: FileId) -> Vec<Arc<DefMap>> {
    db.projet_subgraph(file_id)
        .map(|subgraph| {
            subgraph
                .nodes
                .iter()
                .map(|it| db.file_def_map(it.file_id))
                .collect()
        })
        .unwrap_or_default()
}
