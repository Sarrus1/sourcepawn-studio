mod generated;

use std::marker::PhantomData;

use either::Either;
#[allow(unreachable_pub)]
pub use generated::syntax_kind::SyntaxKind;

use crate::syntax_node::{SyntaxNode, SyntaxNodeChildren, SyntaxToken};

impl From<u16> for SyntaxKind {
    #[inline]
    fn from(d: u16) -> SyntaxKind {
        assert!(d <= (SyntaxKind::__LAST as u16));
        unsafe { std::mem::transmute::<u16, SyntaxKind>(d) }
    }
}

impl From<SyntaxKind> for u16 {
    #[inline]
    fn from(k: SyntaxKind) -> u16 {
        k as u16
    }
}

impl SyntaxKind {
    #[inline]
    pub fn is_trivia(self) -> bool {
        self == SyntaxKind::COMMENT
    }
}

// //! Abstract Syntax Tree, layered on top of untyped `SyntaxNode`s

// mod generated;
// mod traits;
// mod token_ext;
// mod node_ext;
// mod expr_ext;
// mod operators;
// pub mod edit;
// pub mod edit_in_place;
// pub mod make;
// pub mod prec;

// use std::marker::PhantomData;

// use either::Either;

// use crate::{
//     syntax_node::{SyntaxNode, SyntaxNodeChildren, SyntaxToken},
//     SyntaxKind,
// };

// pub use self::{
//     expr_ext::{ArrayExprKind, BlockModifier, CallableExpr, ElseBranch, LiteralKind},
//     generated::{nodes::*, tokens::*},
//     node_ext::{
//         AttrKind, FieldKind, Macro, NameLike, NameOrNameRef, PathSegmentKind, SelfParamKind,
//         SlicePatComponents, StructKind, TraitOrAlias, TypeBoundKind, TypeOrConstParam,
//         VisibilityKind,
//     },
//     operators::{ArithOp, BinaryOp, CmpOp, LogicOp, Ordering, RangeOp, UnaryOp},
//     token_ext::{CommentKind, CommentPlacement, CommentShape, IsString, QuoteOffsets, Radix},
//     traits::{
//         AttrDocCommentIter, DocCommentIter, HasArgList, HasAttrs, HasDocComments, HasGenericParams,
//         HasLoopBody, HasModuleItem, HasName, HasTypeBounds, HasVisibility,
//     },
// };

/// The main trait to go from untyped `SyntaxNode`  to a typed ast. The
/// conversion itself has zero runtime cost: ast and syntax nodes have exactly
/// the same representation: a pointer to the tree root and a pointer to the
/// node itself.
pub trait AstNode {
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxNode;
    fn clone_for_update(&self) -> Self
    where
        Self: Sized,
    {
        Self::cast(self.syntax().clone_for_update()).unwrap()
    }
    fn clone_subtree(&self) -> Self
    where
        Self: Sized,
    {
        Self::cast(self.syntax().clone_subtree()).unwrap()
    }
}

/// Like `AstNode`, but wraps tokens rather than interior nodes.
pub trait AstToken {
    fn can_cast(token: SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxToken) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxToken;

    fn text(&self) -> &str {
        self.syntax().text()
    }
}

/// An iterator over `SyntaxNode` children of a particular AST type.
#[derive(Debug, Clone)]
pub struct AstChildren<N> {
    inner: SyntaxNodeChildren,
    ph: PhantomData<N>,
}

impl<N> AstChildren<N> {
    fn new(parent: &SyntaxNode) -> Self {
        AstChildren {
            inner: parent.children(),
            ph: PhantomData,
        }
    }
}

impl<N: AstNode> Iterator for AstChildren<N> {
    type Item = N;
    fn next(&mut self) -> Option<N> {
        self.inner.find_map(N::cast)
    }
}

impl<L, R> AstNode for Either<L, R>
where
    L: AstNode,
    R: AstNode,
{
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        L::can_cast(kind) || R::can_cast(kind)
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if L::can_cast(syntax.kind()) {
            L::cast(syntax).map(Either::Left)
        } else {
            R::cast(syntax).map(Either::Right)
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        self.as_ref().either(L::syntax, R::syntax)
    }
}

impl<L, R> HasAttrs for Either<L, R>
where
    L: HasAttrs,
    R: HasAttrs,
{
}

mod support {
    use super::{AstChildren, AstNode, SyntaxKind, SyntaxNode, SyntaxToken};

    pub(super) fn child<N: AstNode>(parent: &SyntaxNode) -> Option<N> {
        parent.children().find_map(N::cast)
    }

    pub(super) fn children<N: AstNode>(parent: &SyntaxNode) -> AstChildren<N> {
        AstChildren::new(parent)
    }

    pub(super) fn token(parent: &SyntaxNode, kind: SyntaxKind) -> Option<SyntaxToken> {
        parent
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .find(|it| it.kind() == kind)
    }
}
