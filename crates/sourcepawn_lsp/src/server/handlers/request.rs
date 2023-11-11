use crate::{lsp::from_proto, server::GlobalStateSnapshot};

pub(crate) fn handle_goto_definition(
    snap: GlobalStateSnapshot,
    params: lsp_types::GotoDefinitionParams,
) -> anyhow::Result<Option<lsp_types::GotoDefinitionResponse>> {
    let pos = from_proto::file_position(&snap, params.text_document_position_params)?;

    let links = match snap.analysis.goto_definition(pos)? {
        None => return Ok(None),
        Some(it) => it,
    };
    Ok(Some(lsp_types::GotoDefinitionResponse::Link(links)))
}
