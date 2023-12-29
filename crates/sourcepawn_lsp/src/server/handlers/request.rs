use anyhow::Context;

use crate::{
    lsp::{from_proto, to_proto},
    lsp_ext::SyntaxTreeParams,
    server::GlobalStateSnapshot,
};

pub(crate) fn handle_goto_definition(
    snap: GlobalStateSnapshot,
    params: lsp_types::GotoDefinitionParams,
) -> anyhow::Result<Option<lsp_types::GotoDefinitionResponse>> {
    let pos = from_proto::file_position(&snap, params.text_document_position_params.clone())?;

    let targets = match snap.analysis.goto_definition(pos)? {
        None => return Ok(None),
        Some(it) => it,
    };

    Ok(Some(to_proto::goto_definition_response(&snap, targets)?))
}

pub(crate) fn handle_syntax_tree(
    snap: GlobalStateSnapshot,
    params: SyntaxTreeParams,
) -> anyhow::Result<String> {
    let _tree = snap.analysis.parse(from_proto::file_id(
        &snap,
        &params
            .text_document
            .context("No text_document parameter passed.")?
            .uri,
    )?)?;

    // Ok(prettify_s_expression(&tree.root_node().to_sexp()))
    Ok("".to_string())
}
