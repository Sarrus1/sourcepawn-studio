use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext, Severity};

pub(crate) use self::inactive_code as f;

// Diagnostic: inactive-code
//
// This diagnostic is shown for code with inactive preprocessor directives.
pub(crate) fn inactive_code(_ctx: &DiagnosticsContext<'_>, d: &hir::InactiveCode) -> Diagnostic {
    let message = "code is inactive due to preprocessor directives".to_string();

    // FIXME: This shouldn't be a diagnostic
    Diagnostic::new_for_u_range(
        DiagnosticCode::Lint("inactive-code", Severity::WeakWarning),
        message,
        d.range,
    )
    .with_unused(true)
}
