use std::sync::Arc;

use itertools::Itertools;
use syntax::TSKind;
use vfs::FileId;

use crate::{
    ast_id_map::AstIdMap,
    hir::{type_ref::TypeRef, Expr, ExprId, FloatTypeWrapper, Literal, SwitchCase},
    item_tree::Name,
    BlockLoc, DefDatabase, DefWithBodyId, InFile, NodePtr,
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
    #[allow(unused)]
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
                TSKind::parameter_declarations => {
                    params_list
                        .children(&mut params_list.walk())
                        .filter(|n| TSKind::from(n) == TSKind::parameter_declaration)
                        .for_each(|child| self.collect_parameter_declaration(child));
                }
                TSKind::parameter_declaration => self.collect_parameter_declaration(params_list),
                _ => unreachable!("Expected parameters"),
            }
        }
        if let Some(body) = body {
            self.body.body_expr = self.collect_expr(body).into();
        }
        (self.body, self.source_map)
    }

    fn collect_parameter_declaration(&mut self, node: tree_sitter::Node) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let ident_id = self
                .body
                .idents
                .alloc(Name::from_node(&name_node, self.source));
            let binding = Expr::Binding {
                ident_id,
                type_ref: TypeRef::from_returntype_node(&node, "type", self.source),
                initializer: node
                    .child_by_field_name("initialValue")
                    .map(|default_node| self.collect_expr(default_node)),
            };
            let decl_id = self.alloc_expr(binding, NodePtr::from(&node));
            self.body.params.push((ident_id, decl_id));
        }
    }

    fn collect_variable_declaration(&mut self, expr: tree_sitter::Node) -> ExprId {
        let mut decl = vec![];
        let type_ref = TypeRef::from_returntype_node(&expr, "type", self.source);
        for child in expr.children(&mut expr.walk()).filter(|n| {
            matches!(
                TSKind::from(n),
                TSKind::variable_declaration | TSKind::dynamic_array_declaration
            )
        }) {
            if let Some(name_node) = child.child_by_field_name("name") {
                let ident_id = self
                    .body
                    .idents
                    .alloc(Name::from_node(&name_node, self.source));
                let binding = Expr::Binding {
                    ident_id,
                    type_ref: type_ref.clone(),
                    initializer: child
                        .child_by_field_name("initialValue")
                        .map(|default_node| self.collect_expr(default_node)),
                };
                let binding_id = self.alloc_expr(binding, NodePtr::from(&child));
                decl.push(binding_id);
            }
        }
        let decl = Expr::Decl(decl.into_boxed_slice());
        self.alloc_expr(decl, NodePtr::from(&expr))
    }

    fn collect_old_variable_declaration(&mut self, expr: tree_sitter::Node) -> ExprId {
        let mut decl = vec![];
        for child in expr
            .children(&mut expr.walk())
            .filter(|n| TSKind::from(n) == TSKind::old_variable_declaration)
        {
            let type_ref = TypeRef::from_returntype_node(&child, "type", self.source);
            if let Some(name_node) = child.child_by_field_name("name") {
                let ident_id = self
                    .body
                    .idents
                    .alloc(Name::from_node(&name_node, self.source));
                let binding = Expr::Binding {
                    ident_id,
                    type_ref: type_ref.clone(),
                    initializer: child
                        .child_by_field_name("initialValue")
                        .map(|default_node| self.collect_expr(default_node)),
                };
                let binding_id = self.alloc_expr(binding, NodePtr::from(&child));
                decl.push(binding_id);
            }
        }
        let decl = Expr::Decl(decl.into_boxed_slice());
        self.alloc_expr(decl, NodePtr::from(&expr))
    }

    fn collect_expr(&mut self, expr: tree_sitter::Node) -> ExprId {
        self.maybe_collect_expr(expr)
            .unwrap_or_else(|| self.missing_expr())
    }

    fn maybe_collect_expr(&mut self, expr: tree_sitter::Node) -> Option<ExprId> {
        match TSKind::from(expr) {
            // region: Parameters
            TSKind::named_arg => {
                let name = expr.child_by_field_name("arg_name")?;
                let value = expr.child_by_field_name("value")?;
                let named_arg = Expr::NamedArg {
                    name: self.collect_expr(name),
                    value: self.collect_expr(value),
                };
                Some(self.alloc_expr(named_arg, NodePtr::from(&expr)))
            }
            // endregion: Parameters
            // region: Statements
            TSKind::block => {
                let ast_id = self.ast_id_map.ast_id_of(&expr);
                let block_id = self.db.intern_block(BlockLoc {
                    ast_id,
                    file_id: self.file_id,
                });
                let mut statements = Vec::new();
                for child in expr.children(&mut expr.walk()) {
                    match TSKind::from(child) {
                        TSKind::anon_LBRACE | TSKind::anon_RBRACE => continue,
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
            TSKind::variable_declaration_statement => Some(self.collect_variable_declaration(expr)),
            TSKind::old_variable_declaration_statement
            | TSKind::old_for_loop_variable_declaration_statement => {
                Some(self.collect_old_variable_declaration(expr))
            }
            TSKind::for_statement => {
                let mut initialization = vec![];
                for init in expr.children_by_field_name("initialization", &mut expr.walk()) {
                    initialization.push(self.collect_expr(init));
                }
                let for_loop = Expr::Loop {
                    initialization: initialization.into_boxed_slice(),
                    condition: expr
                        .child_by_field_name("condition")
                        .map(|it| self.collect_expr(it)),
                    iteration: expr
                        .child_by_field_name("iteration")
                        .and_then(|it| self.maybe_collect_expr(it)),
                    body: expr
                        .child_by_field_name("body")
                        .map(|it| self.collect_expr(it)),
                };
                Some(self.alloc_expr(for_loop, NodePtr::from(&expr)))
            }
            TSKind::while_statement | TSKind::do_while_statement => {
                let loop_expr = Expr::Loop {
                    initialization: Default::default(),
                    condition: expr
                        .child_by_field_name("condition")
                        .map(|it| self.collect_expr(it)),
                    iteration: None,
                    body: expr
                        .child_by_field_name("body")
                        .map(|it| self.collect_expr(it)),
                };
                Some(self.alloc_expr(loop_expr, NodePtr::from(&expr)))
            }
            TSKind::break_statement => {
                let control_expr = Expr::Control {
                    keyword: TSKind::anon_break,
                    operand: None,
                };
                Some(self.alloc_expr(control_expr, NodePtr::from(&expr)))
            }
            TSKind::continue_statement => {
                let control_expr = Expr::Control {
                    keyword: TSKind::anon_continue,
                    operand: None,
                };
                Some(self.alloc_expr(control_expr, NodePtr::from(&expr)))
            }
            TSKind::condition_statement => {
                let condition = expr.child_by_field_name("condition")?;
                let then_branch = expr.child_by_field_name("truePath")?;
                let else_branch = expr
                    .child_by_field_name("falsePath")
                    .map(|n| self.collect_expr(n));
                let ternary = Expr::Condition {
                    condition: self.collect_expr(condition),
                    then_branch: self.collect_expr(then_branch),
                    else_branch,
                };
                Some(self.alloc_expr(ternary, NodePtr::from(&expr)))
            }
            TSKind::switch_statement => {
                let condition = expr.child_by_field_name("condition")?;
                let cases = expr
                    .children(&mut expr.walk())
                    .filter(|n| TSKind::from(n) == TSKind::switch_case)
                    .flat_map(|case| {
                        let values = case
                            .children_by_field_name("value", &mut case.walk())
                            .map(|value| self.collect_expr(value))
                            .collect_vec()
                            .into_boxed_slice();
                        let body = case.child_by_field_name("body")?;
                        Some(SwitchCase::new(values, self.collect_expr(body)))
                    })
                    .collect_vec();
                let switch = Expr::Switch {
                    condition: self.collect_expr(condition),
                    cases: cases.into_boxed_slice(),
                };
                Some(self.alloc_expr(switch, NodePtr::from(&expr)))
            }
            TSKind::return_statement => {
                let expr = expr.child_by_field_name("expression")?;
                let control_expr = Expr::Control {
                    keyword: TSKind::anon_return_,
                    operand: self.maybe_collect_expr(expr),
                };
                Some(self.alloc_expr(control_expr, NodePtr::from(&expr)))
            }
            TSKind::delete_statement => {
                let expr = expr.child_by_field_name("free")?;
                let control_expr = Expr::Control {
                    keyword: TSKind::anon_delete_,
                    operand: self.maybe_collect_expr(expr),
                };
                Some(self.alloc_expr(control_expr, NodePtr::from(&expr)))
            }
            TSKind::expression_statement => {
                let child = expr.children(&mut expr.walk()).next()?;
                Some(self.collect_expr(child))
            }
            // endregion: Statements
            // region: Expressions
            TSKind::assignment_expression
            | TSKind::binary_expression
            | TSKind::preproc_binary_expression => {
                let lhs = self.collect_expr(expr.child_by_field_name("left")?);
                let rhs = self.collect_expr(expr.child_by_field_name("right")?);
                let op = expr.child_by_field_name("operator").map(TSKind::from);
                let assign = Expr::BinaryOp { lhs, rhs, op };
                Some(self.alloc_expr(assign, NodePtr::from(&expr)))
            }
            TSKind::call_expression => {
                let function = expr.child_by_field_name("function")?;
                let arguments = expr.child_by_field_name("arguments")?;
                match TSKind::from(&function) {
                    // Function call
                    TSKind::identifier => {
                        let callee = self.collect_expr(function);
                        let args = arguments
                            .children(&mut arguments.walk())
                            .filter_map(|arg| self.maybe_collect_expr(arg))
                            .collect::<Vec<_>>();
                        let call = Expr::Call {
                            callee,
                            args: args.into_boxed_slice(),
                        };
                        Some(self.alloc_expr(call, NodePtr::from(&expr)))
                    }
                    // Method call
                    TSKind::field_access => {
                        let target = function.child_by_field_name("target")?;
                        let method = function.child_by_field_name("field")?;
                        let args = arguments
                            .children(&mut arguments.walk())
                            .filter_map(|arg| self.maybe_collect_expr(arg))
                            .collect::<Vec<_>>();
                        let call = Expr::MethodCall {
                            target: self.collect_expr(target),
                            method_name: Name::from_node(&method, self.source),
                            args: args.into_boxed_slice(),
                        };
                        Some(self.alloc_expr(call, NodePtr::from(&method)))
                    }
                    _ => unreachable!(),
                }
            }
            TSKind::array_indexed_access => {
                let array = expr.child_by_field_name("array")?;
                let index = expr.child_by_field_name("index")?;
                let access = Expr::ArrayIndexedAccess {
                    array: self.collect_expr(array),
                    index: self.collect_expr(index),
                };
                Some(self.alloc_expr(access, NodePtr::from(&expr)))
            }
            TSKind::ternary_expression => {
                let condition = self.collect_expr(expr.child_by_field_name("condition")?);
                let then_branch = self.collect_expr(expr.child_by_field_name("consequence")?);
                let else_branch = self.collect_expr(expr.child_by_field_name("alternative")?);
                let ternary = Expr::TernaryOp {
                    condition,
                    then_branch,
                    else_branch,
                };
                Some(self.alloc_expr(ternary, NodePtr::from(&expr)))
            }
            TSKind::field_access => {
                let field = expr.child_by_field_name("field")?;
                let field_access = Expr::FieldAccess {
                    target: self.collect_expr(expr.child_by_field_name("target")?),
                    name: Name::from_node(&field, self.source),
                };
                Some(self.alloc_expr(field_access, NodePtr::from(&field)))
            }
            TSKind::array_scope_access | TSKind::scope_access => {
                let field = expr.child_by_field_name("field")?;
                let access = Expr::ScopeAccess {
                    scope: self.collect_expr(expr.child_by_field_name("scope")?),
                    field: Name::from_node(&field, self.source),
                };
                Some(self.alloc_expr(access, NodePtr::from(&field)))
            }
            TSKind::unary_expression
            | TSKind::update_expression
            | TSKind::preproc_unary_expression => {
                // For our needs, unary and update expressions are the same
                let expr = expr.child_by_field_name("argument")?;
                let op = expr.child_by_field_name("operator").map(TSKind::from);
                let unary = Expr::UnaryOp {
                    operand: self.collect_expr(expr),
                    op,
                };
                Some(self.alloc_expr(unary, NodePtr::from(&expr)))
            }
            TSKind::sizeof_expression => {
                let type_expr = expr.child_by_field_name("type")?;
                // For Sourcepawn, sizeof as a unary operator will do fine.
                let sizeof = Expr::UnaryOp {
                    operand: self.collect_expr(type_expr),
                    op: Some(TSKind::sizeof_expression),
                };
                Some(self.alloc_expr(sizeof, NodePtr::from(&expr)))
            }
            TSKind::view_as | TSKind::old_type_cast => {
                let value_expr = expr.child_by_field_name("value")?;
                let type_ref = TypeRef::from_returntype_node(&expr, "type", self.source)?;
                let view_as = Expr::ViewAs {
                    operand: self.collect_expr(value_expr),
                    type_ref,
                };
                Some(self.alloc_expr(view_as, NodePtr::from(&expr)))
            }
            TSKind::identifier => {
                let name = Name::from_node(&expr, self.source);
                Some(self.alloc_expr(Expr::Ident(name), NodePtr::from(&expr)))
            }
            TSKind::this => Some(self.alloc_expr(Expr::This, NodePtr::from(&expr))),
            TSKind::int_literal => {
                let text = expr.utf8_text(self.source.as_bytes()).unwrap();
                // FIXME: The unwrap_or_default() is a workaround for hex literals
                let int = text.parse().unwrap_or_default();
                Some(self.alloc_expr(Expr::Literal(Literal::Int(int)), NodePtr::from(&expr)))
            }
            TSKind::float_literal => {
                let text = expr.utf8_text(self.source.as_bytes()).unwrap();
                // FIXME: The unwrap_or_default() is a workaround
                let float = FloatTypeWrapper::new(text.parse().unwrap_or_default());
                Some(self.alloc_expr(Expr::Literal(Literal::Float(float)), NodePtr::from(&expr)))
            }
            TSKind::char_literal => {
                let text = expr.utf8_text(self.source.as_bytes()).unwrap();
                // FIXME: The unwrap_or_default() is a workaround
                let char = text.chars().nth(1).unwrap_or_default();
                Some(self.alloc_expr(Expr::Literal(Literal::Char(char)), NodePtr::from(&expr)))
            }
            TSKind::string_literal => {
                let text = expr.utf8_text(self.source.as_bytes()).unwrap();
                Some(self.alloc_expr(
                    Expr::Literal(Literal::String(text.into())),
                    NodePtr::from(&expr),
                ))
            }
            TSKind::bool_literal => {
                let text = expr.utf8_text(self.source.as_bytes()).unwrap();
                let bool = text.parse().ok()?;
                Some(self.alloc_expr(Expr::Literal(Literal::Bool(bool)), NodePtr::from(&expr)))
            }
            TSKind::null => {
                Some(self.alloc_expr(Expr::Literal(Literal::Null), NodePtr::from(&expr)))
            }
            TSKind::parenthesized_expression | TSKind::preproc_parenthesized_expression => {
                let expr = expr.child_by_field_name("expression")?;
                self.maybe_collect_expr(expr)
            }
            TSKind::new_expression => {
                let constructor = expr.child_by_field_name("class")?;
                let args = expr.child_by_field_name("arguments")?;
                let new = Expr::New {
                    name: Name::from_node(&constructor, self.source),
                    args: args
                        .children(&mut args.walk())
                        .filter_map(|arg| self.maybe_collect_expr(arg))
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                };
                Some(self.alloc_expr(new, NodePtr::from(&constructor)))
            }
            TSKind::dynamic_array => {
                let type_ = expr.child_by_field_name("type")?;
                let dyn_arr = Expr::DynamicArray {
                    identifier: self.maybe_collect_expr(type_)?,
                };
                Some(self.alloc_expr(dyn_arr, NodePtr::from(&expr)))
            }
            TSKind::array_literal => {
                let elements = expr
                    .children(&mut expr.walk())
                    .filter_map(|n| self.maybe_collect_expr(n))
                    .collect_vec()
                    .into_boxed_slice();
                Some(self.alloc_expr(
                    Expr::Literal(Literal::Array(elements)),
                    NodePtr::from(&expr),
                ))
            }
            // endregion: Expressions
            TSKind::comma_expression => {
                let mut exprs = vec![];
                for child in expr.children(&mut expr.walk()) {
                    exprs.push(self.collect_expr(child));
                }
                Some(self.alloc_expr(
                    Expr::CommaExpr(exprs.into_boxed_slice()),
                    NodePtr::from(&expr),
                ))
            }
            TSKind::comment
            | TSKind::anon_LBRACE
            | TSKind::anon_RBRACE
            | TSKind::anon_LBRACK
            | TSKind::anon_RBRACK
            | TSKind::anon_LPAREN
            | TSKind::anon_RPAREN
            | TSKind::anon_COMMA => None,
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
        let ptr = InFile::new(self.file_id, ptr);
        self.source_map.expr_map_back.insert(id, ptr);
        self.source_map.expr_map.insert(ptr, id);
        id
    }
}
