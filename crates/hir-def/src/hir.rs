use la_arena::Idx;
use smallvec::SmallVec;
use syntax::TSKind;

use crate::{item_tree::Name, BlockId};

use self::type_ref::TypeRef;

pub mod type_ref;

pub type Ident = Name;

pub type IdentId = Idx<Ident>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Binding {
    pub name: Name,
    // pub mode: BindingAnnotation,
    pub definitions: SmallVec<[IdentId; 1]>,
    // pub problems: Option<BindingProblems>,
}

pub type ExprId = Idx<Expr>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Expr {
    Missing,
    Ident(Ident),
    Block {
        id: Option<BlockId>,
        statements: Box<[ExprId]>,
    },
    FieldAccess {
        target: ExprId,
        name: Name,
    },
    BinaryOp {
        lhs: ExprId,
        rhs: ExprId,
        op: Option<BinaryOp>,
    },
    Call {
        callee: ExprId,
        args: Box<[ExprId]>,
    },
    MethodCall {
        target: ExprId,
        method_name: Name,
        args: Box<[ExprId]>,
    },
    Decl(Box<[ExprId]>),
    Binding {
        ident_id: IdentId,
        type_ref: Option<TypeRef>,
        initializer: Option<ExprId>,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Assignment { op: Option<TSKind> },
}
