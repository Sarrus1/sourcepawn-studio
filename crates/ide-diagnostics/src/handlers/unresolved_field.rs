use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};

pub(crate) use self::unresolved_field as f;

// Diagnostic: unresolved-field
//
// This diagnostic is triggered if a field does not exist on a given type.
pub(crate) fn unresolved_field(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::UnresolvedField,
) -> Diagnostic {
    let method_suffix = if d.method_with_same_name_exists {
        ", but a method with a similar name exists"
    } else {
        ""
    };
    Diagnostic::new_with_syntax_node_ptr(
        ctx,
        DiagnosticCode::SpCompError("E0000"),
        format!(
            "no field `{}` on type `{}`{method_suffix}",
            d.name, d.receiver
        ),
        d.expr,
    )
    // .with_fixes(fixes(ctx, d))
    // .experimental()
}
