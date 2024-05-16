use std::fmt;

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
    This,
    Block {
        id: Option<BlockId>,
        statements: Box<[ExprId]>,
    },
    #[allow(clippy::enum_variant_names)]
    CommaExpr(Box<[ExprId]>),
    NamedArg {
        name: ExprId,
        value: ExprId,
    },
    New {
        name: Name,
        args: Box<[ExprId]>,
    },
    DynamicArray {
        identifier: ExprId,
    },
    ViewAs {
        operand: ExprId,
        type_ref: TypeRef,
    },
    FieldAccess {
        target: ExprId,
        name: Name,
    },
    ScopeAccess {
        scope: ExprId,
        field: Name,
    },
    ArrayIndexedAccess {
        array: ExprId,
        index: ExprId,
    },
    UnaryOp {
        operand: ExprId,
        op: Option<TSKind>,
    },
    BinaryOp {
        lhs: ExprId,
        rhs: ExprId,
        op: Option<TSKind>,
    },
    TernaryOp {
        condition: ExprId,
        then_branch: ExprId,
        else_branch: ExprId,
    },
    Loop {
        initialization: Box<[ExprId]>,
        condition: ExprId,
        iteration: Option<ExprId>,
        body: ExprId,
    },
    Switch {
        condition: ExprId,
        cases: Box<[SwitchCase]>,
    },
    Condition {
        condition: ExprId,
        then_branch: ExprId,
        else_branch: Option<ExprId>,
    },
    Control {
        keyword: TSKind,
        operand: Option<ExprId>,
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
    Literal(Literal),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SwitchCase {
    values: Box<[ExprId]>,
    body: ExprId,
}

impl SwitchCase {
    pub fn new(values: Box<[ExprId]>, body: ExprId) -> Self {
        Self { values, body }
    }

    pub fn body(&self) -> ExprId {
        self.body
    }

    pub fn values(&self) -> &[ExprId] {
        &self.values
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Literal {
    String(Box<str>),
    Char(char),
    Bool(bool),
    Int(i64),
    Null,
    // Here we are using a wrapper around float because f32 and f64 do not implement Eq, so they
    // could not be used directly here, to understand how the wrapper works go to definition of
    // FloatTypeWrapper
    Float(FloatTypeWrapper),
    Array(Box<[ExprId]>),
}

// We convert float values into bits and that's how we don't need to deal with f32 and f64.
// For PartialEq, bits comparison should work, as ordering is not important
// https://github.com/rust-lang/rust-analyzer/issues/12380#issuecomment-1137284360
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct FloatTypeWrapper(u64);

impl FloatTypeWrapper {
    pub fn new(value: f64) -> Self {
        Self(value.to_bits())
    }

    pub fn into_f64(self) -> f64 {
        f64::from_bits(self.0)
    }

    pub fn into_f32(self) -> f32 {
        f64::from_bits(self.0) as f32
    }
}

impl fmt::Display for FloatTypeWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", f64::from_bits(self.0))
    }
}
