use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};

pub(crate) use self::preprocessor_evaluation_error as f;

// Diagnostic: unresolved-field
//
// This diagnostic is triggered if a field does not exist on a given type.
pub(crate) fn preprocessor_evaluation_error(
    _ctx: &DiagnosticsContext<'_>,
    d: &hir::PreprocessorEvaluationError,
) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::SpCompError("E0000"),
        d.text.to_owned(),
        d.range,
    )
    // .with_fixes(fixes(ctx, d))
    // .experimental()
}
