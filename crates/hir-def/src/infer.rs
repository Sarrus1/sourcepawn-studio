use std::sync::Arc;

use fxhash::FxHashMap;

use crate::{
    body::Body,
    data::{EnumStructItemData, MethodmapItemData},
    hir::{type_ref::TypeRef, BinaryOp, Expr},
    item_tree::Name,
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InferenceDiagnostic {
    UnresolvedField {
        expr: ExprId,
        receiver: Name,
        name: Name,
        method_with_same_name_exists: bool,
    },
    UnresolvedMethodCall {
        expr: ExprId,
        receiver: Name,
        name: Name,
        field_with_same_name_exists: bool,
    },
}

/// The result of type inference: A mapping from expressions and patterns to types.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct InferenceResult {
    /// For each field access expr, records the field it resolves to.
    field_resolutions: FxHashMap<ExprId, FieldId>,
    /// For each method call expr, records the function it resolves to.
    method_resolutions: FxHashMap<ExprId, FunctionId>,

    pub diagnostics: Vec<InferenceDiagnostic>,
}

impl InferenceResult {
    pub fn field_resolution(&self, expr: ExprId) -> Option<FieldId> {
        self.field_resolutions.get(&expr).copied()
    }

    pub fn method_resolution(&self, expr: ExprId) -> Option<FunctionId> {
        self.method_resolutions.get(&expr).copied()
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
            Expr::FieldAccess { target, name } => self.infer_field_access(expr, target, name),
            Expr::BinaryOp { lhs, rhs, op } => {
                let _lhs_ty = self.infer_expr(lhs);
                let _rhs_ty = self.infer_expr(rhs);
                match op.as_ref()? {
                    BinaryOp::Assignment { op: _ } => None,
                }
            }
            Expr::Ident(name) => {
                let name: String = name.clone().into();
                let res = self.resolver.resolve_ident(&name)?; // TODO: Should we emit a diagnostic here?
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
                    ValueNs::MethodmapId(it) => {
                        let item_tree = self.db.file_item_tree(it.file_id);
                        TypeRef::Name(item_tree[it.value.lookup(self.db).id].name.clone()).into()
                    }
                    ValueNs::EnumStructId(it) => {
                        let item_tree = self.db.file_item_tree(it.file_id);
                        TypeRef::Name(item_tree[it.value.lookup(self.db).id].name.clone()).into()
                    }
                    _ => todo!(),
                }
            }
            Expr::MethodCall {
                target,
                method_name,
                args,
            } => self.infer_method_call(expr, target, method_name, args),
            Expr::Call { callee, args } => {
                for arg in args.iter() {
                    self.infer_expr(arg);
                }
                let Expr::Ident(callee) = &self.body[*callee] else {
                    panic!("Callees are identifiers.")
                };
                let name: String = callee.clone().into();
                match self.resolver.resolve_ident(&name)? {
                    ValueNs::FunctionId(it) => {
                        let item_tree = self.db.file_item_tree(it.file_id);
                        item_tree[it.value.lookup(self.db).id.value]
                            .ret_type
                            .clone()
                    }
                    _ => todo!(),
                }
            }
            Expr::Missing | Expr::Decl(_) | Expr::Binding { .. } => None,
        }
    }

    pub(crate) fn collect_fn(&mut self, _func: FunctionId) {
        if let Some(id) = self.body.body_expr {
            self.infer_expr(&id);
        }
    }

    fn infer_field_access(
        &mut self,
        receiver: &ExprId,
        target: &ExprId,
        name: &Name,
    ) -> Option<TypeRef> {
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
        if let Some(item) = data.items(name) {
            match data.item(item) {
                EnumStructItemData::Field(_) => {
                    let field_id = FieldId {
                        parent: it,
                        local_id: item,
                    };
                    self.result.field_resolutions.insert(*receiver, field_id);
                    return Some(data.field_type(item)?.clone());
                }
                EnumStructItemData::Method(_) => {
                    self.result
                        .diagnostics
                        .push(InferenceDiagnostic::UnresolvedField {
                            expr: *receiver,
                            receiver: type_name,
                            name: name.clone(),
                            method_with_same_name_exists: true,
                        });
                    return None;
                }
            }
        }
        self.result
            .diagnostics
            .push(InferenceDiagnostic::UnresolvedField {
                expr: *receiver,
                receiver: type_name,
                name: name.clone(),
                method_with_same_name_exists: false,
            });

        None
    }

    fn infer_method_call(
        &mut self,
        receiver: &ExprId,
        target: &ExprId,
        method_name: &Name,
        args: &[ExprId],
    ) -> Option<TypeRef> {
        for arg in args.iter() {
            self.infer_expr(arg);
        }

        let target_ty = self.infer_expr(target);
        let Some(TypeRef::Name(type_name)) = target_ty else {
            return None;
        };
        let def_map = self.db.file_def_map(self.owner.file_id(self.db));
        match def_map.get(&type_name)? {
            FileDefId::EnumStructId(it) => {
                let data = self.db.enum_struct_data(it);
                if let Some(item) = data.items(method_name) {
                    match data.item(item) {
                        EnumStructItemData::Field(_) => {
                            self.result.diagnostics.push(
                                InferenceDiagnostic::UnresolvedMethodCall {
                                    expr: *receiver,
                                    receiver: type_name,
                                    name: method_name.clone(),
                                    field_with_same_name_exists: true,
                                },
                            );
                            return None;
                        }
                        EnumStructItemData::Method(method) => {
                            self.result.method_resolutions.insert(*receiver, *method);
                            let function = method.lookup(self.db);
                            let item_tree = function.id.item_tree(self.db);
                            return item_tree[function.id.value].ret_type.clone();
                        }
                    }
                }
            }
            FileDefId::MethodmapId(it) => {
                let data = self.db.methodmap_data(it);
                if let Some(item) = data.items(method_name) {
                    match data.item(item) {
                        MethodmapItemData::Property(_) => {
                            self.result.diagnostics.push(
                                InferenceDiagnostic::UnresolvedMethodCall {
                                    expr: *receiver,
                                    receiver: type_name,
                                    name: method_name.clone(),
                                    field_with_same_name_exists: true,
                                },
                            );
                            return None;
                        }
                        MethodmapItemData::Method(method) => {
                            self.result.method_resolutions.insert(*receiver, *method);
                            let function = method.lookup(self.db);
                            let item_tree = function.id.item_tree(self.db);
                            return item_tree[function.id.value].ret_type.clone();
                        }
                    }
                }
            }
            _ => unreachable!("Method calls are only allowed on enum structs and methodmaps."),
        }
        self.result
            .diagnostics
            .push(InferenceDiagnostic::UnresolvedMethodCall {
                expr: *receiver,
                receiver: type_name,
                name: method_name.clone(),
                field_with_same_name_exists: false,
            });

        None
    }
}
