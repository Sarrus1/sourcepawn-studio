//! Re-export diagnostics such that clients of `hir` don't have to depend on
//! low-level crates.
//!
//! This probably isn't the best way to do this -- ideally, diagnostics should
//! be expressed in terms of hir types themselves.

use hir_def::{InFile, Name, NodePtr};

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
    PreprocessorEvaluationError,
    InactiveCode,
];

#[derive(Debug)]
pub struct UnresolvedInclude {
    pub range: lsp_types::Range,
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
pub struct PreprocessorEvaluationError {
    pub range: lsp_types::Range,
    pub text: String,
}

#[derive(Debug)]
pub struct InactiveCode {
    pub range: lsp_types::Range,
}
