use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};

pub(crate) use self::unresolved_include as f;

// Diagnostic: unresolved-include
//
// This diagnostic is triggered if an include is not found.
pub(crate) fn unresolved_include(
    _ctx: &DiagnosticsContext<'_>,
    d: &hir::UnresolvedInclude,
) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::SpCompError("E0000"),
        format!("file `{}` was not found", d.path),
        d.range,
    )
    // .with_fixes(fixes(ctx, d))
    // .experimental()
}
