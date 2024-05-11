use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};

pub(crate) use self::unresolved_macro as f;

// Diagnostic: unresolved-macro
//
// This diagnostic is triggered if a macro is unresolved in a preprocessing directive (#if).
pub(crate) fn unresolved_macro(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::UnresolvedMacro,
) -> Diagnostic {
    Diagnostic::new_for_s_range(
        ctx,
        DiagnosticCode::SpCompError("E0000"),
        format!("no macro `{}` found", d.name),
        d.range,
    )
    // .with_fixes(fixes(ctx, d))
    // .experimental()
}
