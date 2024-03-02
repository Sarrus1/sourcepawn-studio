use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};

pub(crate) use self::unresolved_named_arg as f;

// Diagnostic: unresolved-named-argument
//
// This diagnostic is triggered if a named argument is not found.
pub(crate) fn unresolved_named_arg(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::UnresolvedNamedArg,
) -> Diagnostic {
    Diagnostic::new_with_syntax_node_ptr(
        ctx,
        DiagnosticCode::SpCompError("E0000"),
        format!("no parameter `{}` found", d.name),
        d.expr,
    )
}
