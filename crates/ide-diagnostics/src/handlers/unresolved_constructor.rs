use hir::ConstructorDiagnosticKind;

use crate::{Diagnostic, DiagnosticCode, DiagnosticsContext};

pub(crate) use self::unresolved_constructor as f;

// Diagnostic: unresolved-constructor
//
// This diagnostic is triggered if a constructor call cannot be resolved.
pub(crate) fn unresolved_constructor(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::UnresolvedConstructor,
) -> Diagnostic {
    let message = match d.exists {
        Some(ConstructorDiagnosticKind::Methodmap) => {
            format!("methodmap `{}` does not have a constructor", d.methodmap)
        }
        Some(ConstructorDiagnosticKind::EnumStruct) => {
            format!("enum struct `{}` found, expected a methodmap", d.methodmap)
        }
        None => format!("methodmap `{}` does not exist", d.methodmap),
    };
    Diagnostic::new_with_syntax_node_ptr(ctx, DiagnosticCode::SpCompError("E0000"), message, d.expr)
    // .with_fixes(fixes(ctx, d))
    // .experimental()
}
