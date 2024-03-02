use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};

pub(crate) use self::incorrect_number_of_arguments as f;

// Diagnostic: incorrect-number-of-arguments
//
// This diagnostic is triggered if a call has too few or too many arguments.
pub(crate) fn incorrect_number_of_arguments(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::IncorrectNumberOfArguments,
) -> Diagnostic {
    let message = if d.expected > d.actual {
        format!(
            "expected at least {} arguments for `{}` call, found {}",
            d.expected, d.name, d.actual
        )
    } else {
        format!(
            "expected at most {} arguments for `{}` call, found {}",
            d.expected, d.name, d.actual
        )
    };
    Diagnostic::new_with_syntax_node_ptr(ctx, DiagnosticCode::SpCompError("E0000"), message, d.expr)
}
