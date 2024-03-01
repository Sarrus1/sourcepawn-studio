use std::sync::Arc;

use fxhash::FxHashMap;
use stdx::impl_from;

use crate::{
    body::Body,
    data::{EnumStructItemData, MethodmapItemData},
    hir::{type_ref::TypeRef, Expr, Literal},
    item_tree::Name,
    resolver::{HasResolver, Resolver, ValueNs},
    DefDatabase, DefWithBodyId, ExprId, FieldId, FunctionId, Lookup, PropertyId,
};

pub(crate) fn infer_query(db: &dyn DefDatabase, def: DefWithBodyId) -> Arc<InferenceResult> {
    let body = db.body(def);
    let resolver = def.resolver(db);
    let mut ctx = InferenceContext::new(db, def, &body, resolver);
    match def {
        DefWithBodyId::FunctionId(it) => {
            ctx.collect_fn(it);
        }
        DefWithBodyId::TypedefId(_) | DefWithBodyId::FunctagId(_) => (),
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
    UnresolvedConstructor {
        expr: ExprId,
        methodmap: Name,
        exists: Option<ConstructorDiagnosticKind>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ConstructorDiagnosticKind {
    EnumStruct,
    Methodmap,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AttributeId {
    FieldId(FieldId),
    PropertyId(PropertyId),
}

impl_from!(FieldId, PropertyId for AttributeId);

/// The result of type inference: A mapping from expressions and patterns to types.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct InferenceResult {
    /// For each field/property access expr, records the field/property it resolves to.
    attribute_resolutions: FxHashMap<ExprId, AttributeId>,
    /// For each method call expr, records the function it resolves to.
    method_resolutions: FxHashMap<ExprId, FunctionId>,

    pub diagnostics: Vec<InferenceDiagnostic>,
}

impl InferenceResult {
    pub fn attribute_resolution(&self, expr: ExprId) -> Option<AttributeId> {
        self.attribute_resolutions.get(&expr).copied()
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
            Expr::CommaExpr(exprs) => {
                let mut ty = None;
                for expr in exprs.iter() {
                    ty = self.infer_expr(expr);
                }
                ty
            }
            Expr::New { name, args } => {
                for arg in args.iter() {
                    self.infer_expr(arg);
                }
                self.infer_constructor(expr, name)
            }
            Expr::FieldAccess { target, name } => self.infer_field_access(expr, target, name),
            Expr::UnaryOp { expr, .. } => self.infer_expr(expr),
            Expr::BinaryOp { lhs, rhs, .. } => {
                let _ = self.infer_expr(lhs);
                // Assume the type of the left-hand side is the same as the right-hand side.
                self.infer_expr(rhs)
            }
            Expr::TernaryOp {
                condition,
                then_branch,
                else_branch,
            } => {
                self.infer_expr(condition);
                let _ = self.infer_expr(then_branch);
                // Assume the type of the then branch is the same as the else branch.
                self.infer_expr(else_branch)
            }
            Expr::ScopeAccess { scope, field } => self.infer_field_access(expr, scope, field),
            Expr::ViewAs { expr, type_ref } => {
                let _ = self.infer_expr(expr);
                Some(type_ref.clone())
            }
            Expr::Literal(lit) => {
                let ty = match lit {
                    Literal::Int(_) => TypeRef::Int,
                    Literal::Bool(_) => TypeRef::Bool,
                    Literal::Float(_) => TypeRef::Float,
                    Literal::Char(_) => TypeRef::Char,
                    Literal::String(_) => TypeRef::OldString,
                    Literal::Null => TypeRef::Void,
                };
                Some(ty)
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
                    ValueNs::FunctionId(it) => {
                        let mut ret_type = None;
                        for fn_id in it.iter() {
                            let item_tree = self.db.file_item_tree(fn_id.file_id);
                            let function = &item_tree[fn_id.value.lookup(self.db).id];
                            ret_type = function.ret_type.clone();
                        }

                        ret_type
                    }
                    ValueNs::TypedefId(it) => {
                        let item_tree = self.db.file_item_tree(it.file_id);
                        let name = item_tree[it.value.lookup(self.db).id].name.clone()?;
                        TypeRef::Name(name).into()
                    }
                    ValueNs::TypesetId(it) => {
                        let item_tree = self.db.file_item_tree(it.file_id);
                        TypeRef::Name(item_tree[it.value.lookup(self.db).id].name.clone()).into()
                    }
                    ValueNs::FunctagId(it) => {
                        let item_tree = self.db.file_item_tree(it.file_id);
                        let name = item_tree[it.value.lookup(self.db).id].name.clone()?;
                        TypeRef::Name(name).into()
                    }
                    ValueNs::FuncenumId(it) => {
                        let item_tree = self.db.file_item_tree(it.file_id);
                        TypeRef::Name(item_tree[it.value.lookup(self.db).id].name.clone()).into()
                    }
                    ValueNs::VariantId(_) | ValueNs::EnumId(_) | ValueNs::MacroId(_) => None,
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
                self.infer_expr(callee)
            }
            Expr::Missing | Expr::Decl(_) | Expr::Binding { .. } => None,
        }
    }

    pub(crate) fn collect_fn(&mut self, _func: FunctionId) {
        if let Some(id) = self.body.body_expr {
            self.infer_expr(&id);
        }
    }

    fn infer_constructor(&mut self, expr: &ExprId, name: &Name) -> Option<TypeRef> {
        let type_name_str: String = name.clone().into();
        match self.resolver.resolve_ident(&type_name_str) {
            Some(ValueNs::EnumStructId(_)) => {
                self.result
                    .diagnostics
                    .push(InferenceDiagnostic::UnresolvedConstructor {
                        expr: *expr,
                        methodmap: name.clone(),
                        exists: Some(ConstructorDiagnosticKind::EnumStruct),
                    });
                None
            }
            Some(ValueNs::MethodmapId(it)) => {
                let data = self.db.methodmap_data(it.value);
                if let Some(constructor_id) = data.constructor() {
                    self.result.method_resolutions.insert(*expr, constructor_id);
                    TypeRef::Name(name.clone()).into()
                } else {
                    self.result
                        .diagnostics
                        .push(InferenceDiagnostic::UnresolvedConstructor {
                            expr: *expr,
                            methodmap: name.clone(),
                            exists: Some(ConstructorDiagnosticKind::Methodmap),
                        });
                    None
                }
            }
            _ => {
                self.result
                    .diagnostics
                    .push(InferenceDiagnostic::UnresolvedConstructor {
                        expr: *expr,
                        methodmap: name.clone(),
                        exists: None,
                    });
                None
            }
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
        let type_name_str: String = type_name.clone().into();
        match self.resolver.resolve_ident(&type_name_str)? {
            ValueNs::EnumStructId(it) => {
                let data = self.db.enum_struct_data(it.value);
                if let Some(item) = data.items(name) {
                    match data.item(item) {
                        EnumStructItemData::Field(_) => {
                            let field_id = FieldId {
                                parent: it.value,
                                local_id: item,
                            };
                            self.result
                                .attribute_resolutions
                                .insert(*receiver, field_id.into());
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
            }
            ValueNs::MethodmapId(it) => {
                let data = self.db.methodmap_data(it.value);
                if let Some(item) = data.items(name) {
                    match data.item(item) {
                        MethodmapItemData::Property(_) => {
                            let property_id = PropertyId {
                                parent: it.value,
                                local_id: item,
                            };
                            self.result
                                .attribute_resolutions
                                .insert(*receiver, property_id.into());
                            return Some(data.property_type(item)?.clone());
                        }
                        MethodmapItemData::Method(_)
                        | MethodmapItemData::Constructor(_)
                        | MethodmapItemData::Destructor(_) => {
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
            }
            _ => unreachable!("Field access is only allowed on enum structs and methodmaps."),
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
        let type_name_str: String = type_name.clone().into();
        match self.resolver.resolve_ident(&type_name_str)? {
            ValueNs::EnumStructId(it) => {
                let data = self.db.enum_struct_data(it.value);
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
            ValueNs::MethodmapId(it) => {
                let data = self.db.methodmap_data(it.value);
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
                        MethodmapItemData::Method(method)
                        | MethodmapItemData::Constructor(method)
                        | MethodmapItemData::Destructor(method) => {
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
