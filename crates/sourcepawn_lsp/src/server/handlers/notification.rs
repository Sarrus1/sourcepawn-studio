use lsp_types::{
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
};

use crate::{
    capabilities::ClientCapabilitiesExt, config::Config, lsp::utils::apply_document_changes,
    GlobalState,
};

pub(crate) fn handle_did_change_text_document(
    state: &mut GlobalState,
    params: DidChangeTextDocumentParams,
) -> anyhow::Result<()> {
    let uri = params.text_document.uri;

    let text = apply_document_changes(
        state.config.position_encoding(),
        || {
            let vfs = &state.vfs.read();
            let file_id = vfs.file_id(&uri).unwrap();
            std::str::from_utf8(vfs.file_contents(file_id))
                .unwrap()
                .into()
        },
        params.content_changes,
    );
    state
        .vfs
        .write()
        .set_file_contents(uri, Some(text.into_bytes()));

    Ok(())
}

pub(crate) fn handle_did_open_text_document(
    state: &mut GlobalState,
    params: DidOpenTextDocumentParams,
) -> anyhow::Result<()> {
    let uri = params.text_document.uri;

    state
        .vfs
        .write()
        .set_file_contents(uri, Some(params.text_document.text.into_bytes()));

    Ok(())
}

pub(crate) fn handle_did_change_configuration(
    state: &mut GlobalState,
    _params: DidChangeConfigurationParams,
) -> anyhow::Result<()> {
    // As stated in https://github.com/microsoft/language-server-protocol/issues/676,
    // this notification's parameters should be ignored and the actual config queried separately.
    if !state.config.caps().has_pull_configuration_support() {
        log::trace!("Client does not have pull configuration support.");
        return Ok(());
    }

    state.send_request::<lsp_types::request::WorkspaceConfiguration>(
        lsp_types::ConfigurationParams {
            items: vec![lsp_types::ConfigurationItem {
                section: Some(
                    if state.amxxpawn_mode {
                        "AMXXPawnLanguageServer"
                    } else {
                        "SourcePawnLanguageServer"
                    }
                    .to_string(),
                ),
                scope_uri: None,
            }],
        },
        |this, resp| {
            tracing::debug!("config update response: '{:?}", resp);
            let lsp_server::Response { error, result, .. } = resp;

            match (error, result) {
                (Some(err), _) => {
                    tracing::error!("failed to fetch the server settings: {:?}", err)
                }
                (None, Some(mut configs)) => {
                    if let Some(json) = configs.get_mut(0) {
                        // Note that json can be null according to the spec if the client can't
                        // provide a configuration. This is handled in Config::update below.
                        let mut config = Config::clone(&*this.config);
                        this.config_errors = config.update(json.take()).err();
                        this.update_configuration(config);
                    }
                }
                (None, None) => {
                    tracing::error!("received empty server settings response from the client")
                }
            }
        },
    );

    Ok(())
}
