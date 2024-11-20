//! Re-export diagnostics such that clients of `hir` don't have to depend on
//! low-level crates.
//!
//! This probably isn't the best way to do this -- ideally, diagnostics should
//! be expressed in terms of hir types themselves.

use hir_def::{InFile, Name, NodePtr};
use sourcepawn_lexer::TextRange;

macro_rules! diagnostics {
    ($($diag:ident,)*) => {
        #[derive(Debug)]
        pub enum AnyDiagnostic {$(
            $diag(Box<$diag>),
        )*}

        $(
            impl From<$diag> for AnyDiagnostic {
                fn from(d: $diag) -> AnyDiagnostic {
                    AnyDiagnostic::$diag(Box::new(d))
                }
            }
        )*
    };
}

diagnostics![
    UnresolvedInclude,
    UnresolvedField,
    UnresolvedMethodCall,
    UnresolvedConstructor,
    UnresolvedNamedArg,
    IncorrectNumberOfArguments,
    UnresolvedInherit,
    PreprocessorEvaluationError,
    UnresolvedMacro,
    InactiveCode,
    InvalidUseOfThis,
];

#[derive(Debug)]
pub struct UnresolvedInclude {
    pub range: TextRange,
    pub path: String,
}

#[derive(Debug)]
pub struct UnresolvedField {
    pub expr: InFile<NodePtr>,
    pub receiver: Name,
    pub name: Name,
    pub method_with_same_name_exists: bool,
}

#[derive(Debug)]
pub struct UnresolvedMethodCall {
    pub expr: InFile<NodePtr>,
    pub receiver: Name,
    pub name: Name,
    pub field_with_same_name_exists: bool,
}

#[derive(Debug)]
pub struct UnresolvedConstructor {
    pub expr: InFile<NodePtr>,
    pub methodmap: Name,
    pub exists: Option<ConstructorDiagnosticKind>,
}

#[derive(Debug)]
pub struct InvalidUseOfThis {
    pub expr: InFile<NodePtr>,
}

#[derive(Debug)]
pub enum ConstructorDiagnosticKind {
    Methodmap,
    EnumStruct,
}

#[derive(Debug)]
pub struct UnresolvedNamedArg {
    pub expr: InFile<NodePtr>,
    pub name: Name,
    pub callee: Name,
}

#[derive(Debug)]
pub struct IncorrectNumberOfArguments {
    pub expr: InFile<NodePtr>,
    pub name: Name,
    pub expected: usize,
    pub actual: usize,
}

#[derive(Debug)]
pub struct UnresolvedInherit {
    pub expr: InFile<NodePtr>,
    pub inherit: Name,
    pub exists: bool,
}

#[derive(Debug)]
pub struct PreprocessorEvaluationError {
    pub range: TextRange,
    pub text: String,
}

#[derive(Debug)]
pub struct UnresolvedMacro {
    pub range: TextRange,
    pub name: String,
}

#[derive(Debug)]
pub struct InactiveCode {
    pub range: TextRange,
}
