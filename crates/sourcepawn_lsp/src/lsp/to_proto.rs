use ide::{Cancellable, NavigationTarget};

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
