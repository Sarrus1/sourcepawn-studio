use la_arena::Idx;
use smallvec::SmallVec;

use crate::{item_tree::Name, BlockId};

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
    Ident(Ident),
    Block {
        id: Option<BlockId>,
        statements: Box<[ExprId]>,
    },
    Binding,
    Decl(Vec<(IdentId, ExprId, Option<ExprId>)>), // (IdentId, Option<ExprId>)>), // type_ref: Option<Interned<TypeRef>>,
}
