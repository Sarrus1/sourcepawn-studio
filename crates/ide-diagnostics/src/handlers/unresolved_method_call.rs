use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};

pub(crate) use self::unresolved_method_call as f;

// Diagnostic: unresolved-method-call
//
// This diagnostic is triggered if a method does not exist on a given type.
pub(crate) fn unresolved_method_call(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::UnresolvedMethodCall,
) -> Diagnostic {
    let field_suffix = if d.field_with_same_name_exists {
        ", but a field with a similar name exists"
    } else {
        ""
    };
    Diagnostic::new_with_syntax_node_ptr(
        ctx,
        DiagnosticCode::SpCompError("E0000"),
        format!(
            "no method `{}` on type `{}`{field_suffix}",
            d.name, d.receiver
        ),
        d.expr,
    )
    // .with_fixes(fixes(ctx, d))
    // .experimental()
}
