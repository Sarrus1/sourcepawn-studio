use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};

pub(crate) use self::invalid_use_of_this as f;

// Diagnostic: invalid-use-of-this
//
// This diagnostic is triggered if `this` is used outside of a method.
pub(crate) fn invalid_use_of_this(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::InvalidUseOfThis,
) -> Diagnostic {
    Diagnostic::new_with_syntax_node_ptr(
        ctx,
        DiagnosticCode::SpCompError("E0000"),
        "`this` can only be used in methods",
        d.expr,
    )
    // .with_fixes(fixes(ctx, d))
    // .experimental()
}
