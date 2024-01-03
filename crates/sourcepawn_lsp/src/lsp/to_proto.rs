use ide::{Cancellable, NavigationTarget, Severity};

use crate::server::GlobalStateSnapshot;

pub(crate) fn goto_definition_response(
    snap: &GlobalStateSnapshot,
    targets: Vec<NavigationTarget>,
) -> Cancellable<lsp_types::GotoDefinitionResponse> {
    Ok(lsp_types::GotoDefinitionResponse::Link(
        targets
            .into_iter()
            .map(|target| lsp_types::LocationLink {
                target_uri: snap.file_id_to_url(target.file_id),
                origin_selection_range: Some(target.origin_selection_range),
                target_range: target.target_range,
                target_selection_range: target.target_selection_range,
            })
            .collect(),
    ))
}

pub(crate) fn diagnostic_severity(severity: Severity) -> lsp_types::DiagnosticSeverity {
    match severity {
        Severity::Error => lsp_types::DiagnosticSeverity::ERROR,
        Severity::Warning => lsp_types::DiagnosticSeverity::WARNING,
    }
}
