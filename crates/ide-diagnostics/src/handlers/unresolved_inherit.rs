use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};

pub(crate) use self::unresolved_inherit as f;

// Diagnostic: unresolved-inherit
//
// This diagnostic is triggered if a methodmap inherit is unresolved or resolves to an incorrect type.
pub(crate) fn unresolved_inherit(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::UnresolvedInherit,
) -> Diagnostic {
    let message = if d.exists {
        format!("methodmap `{}` does not exist", d.inherit)
    } else {
        format!("`{}` is not a methodmap", d.inherit)
    };
    Diagnostic::new_with_syntax_node_ptr(ctx, DiagnosticCode::SpCompError("E0000"), message, d.expr)
}
