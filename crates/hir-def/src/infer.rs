use std::sync::Arc;

use fxhash::FxHashMap;

use crate::{
    body::Body,
    hir::{type_ref::TypeRef, BinaryOp, Expr},
    resolver::{HasResolver, Resolver, ValueNs},
    DefDatabase, DefWithBodyId, ExprId, FieldId, FileDefId, FunctionId, Lookup,
};

pub(crate) fn infer_query(db: &dyn DefDatabase, def: DefWithBodyId) -> Arc<InferenceResult> {
    let body = db.body(def);
    let resolver = def.resolver(db); // FIXME: The resolver is not properly initialized yet...
    let mut ctx = InferenceContext::new(db, def, &body, resolver);
    match def {
        DefWithBodyId::FunctionId(it) => {
            ctx.collect_fn(it);
        }
    }

    Arc::new(ctx.result)
}

/// The result of type inference: A mapping from expressions and patterns to types.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct InferenceResult {
    /// For each field access expr, records the field it resolves to.
    field_resolutions: FxHashMap<ExprId, FieldId>,
}

impl InferenceResult {
    pub fn field_resolution(&self, expr: ExprId) -> Option<FieldId> {
        self.field_resolutions.get(&expr).copied()
    }
}

/// The inference context contains all information needed during type inference.
#[derive(Clone, Debug)]
pub(crate) struct InferenceContext<'a> {
    pub(crate) db: &'a dyn DefDatabase,
    pub(crate) owner: DefWithBodyId,
    pub(crate) body: &'a Body,
    pub(crate) result: InferenceResult,
    pub(crate) resolver: Resolver,
}

impl<'a> InferenceContext<'a> {
    fn new(
        db: &'a dyn DefDatabase,
        owner: DefWithBodyId,
        body: &'a Body,
        resolver: Resolver,
    ) -> Self {
        InferenceContext {
            result: InferenceResult::default(),
            db,
            owner,
            body,
            resolver,
        }
    }
}

impl InferenceContext<'_> {
    pub(crate) fn infer_expr(&mut self, expr: &ExprId) -> Option<TypeRef> {
        match &self.body[*expr] {
            Expr::Block { id: _, statements } => {
                let g = self
                    .resolver
                    .update_to_inner_scope(self.db, self.owner, *expr);
                for expr_id in statements.iter() {
                    self.infer_expr(expr_id);
                }
                self.resolver.reset_to_guard(g);
                None
            }
            Expr::FieldAccess { target, name } => {
                let target_ty = self.infer_expr(target);
                let Some(TypeRef::Name(type_name)) = target_ty else {
                    return None;
                };
                let def_map = self.db.file_def_map(self.owner.file_id(self.db));
                let res = def_map.get(&type_name)?;
                let FileDefId::EnumStructId(it) = res else {
                    return None;
                };
                let data = self.db.enum_struct_data(it);
                let field = data.field(name)?;
                let field_id = FieldId {
                    parent: it,
                    local_id: field,
                };
                self.result.field_resolutions.insert(*expr, field_id);
                return Some(data.field_type(field).clone());
            }
            Expr::BinaryOp { lhs, rhs, op } => {
                let _lhs_ty = self.infer_expr(lhs);
                let _rhs_ty = self.infer_expr(rhs);
                match op.as_ref()? {
                    BinaryOp::Assignment { op: _ } => None,
                }
            }
            Expr::Ident(name) => {
                let name: String = name.clone().into();
                let res = self.resolver.resolve_ident(name.as_str())?;
                match res {
                    ValueNs::GlobalId(it) => {
                        let item_tree = self.db.file_item_tree(it.file_id);
                        item_tree[it.value.lookup(self.db).value].type_ref.clone()
                    }
                    ValueNs::LocalId((_, expr_id)) => {
                        let Expr::Binding {
                            ident_id: _,
                            type_ref,
                            initializer: _,
                        } = &self.body[expr_id]
                        else {
                            return None;
                        };
                        type_ref.as_ref().cloned()
                    }
                    _ => todo!(),
                }
            }
            Expr::Missing | Expr::Decl(_) | Expr::Binding { .. } => None,
        }
    }

    pub(crate) fn collect_fn(&mut self, _func: FunctionId) {
        self.infer_expr(&self.body.body_expr);
    }
}
