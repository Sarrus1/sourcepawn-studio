use std::panic::AssertUnwindSafe;

use itertools::Itertools;
use lsp_types::{
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    WorkDoneProgressCancelParams,
};
use salsa::Cancelled;
use vfs::{FileId, VfsPath};

use crate::{
    capabilities::ClientCapabilitiesExt,
    config::Config,
    lsp::{from_proto, utils::apply_document_changes},
    mem_docs::DocumentData,
    GlobalState,
};

pub(crate) fn handle_did_change_text_document(
    state: &mut GlobalState,
    params: DidChangeTextDocumentParams,
) -> anyhow::Result<()> {
    log::debug!(
        "Handling did change text document {}",
        params.text_document.uri
    );
    if let Ok(path) = from_proto::vfs_path(&params.text_document.uri) {
        match state.mem_docs.get_mut(&path) {
            Some(doc) => {
                // The version passed in DidChangeTextDocument is the version after all edits are applied
                // so we should apply it before the vfs is notified.
                doc.version = params.text_document.version;
            }
            None => {
                log::error!("unexpected DidChangeTextDocument: {}", path);
                return Ok(());
            }
        };

        let text = apply_document_changes(
            state.config.position_encoding(),
            || {
                let vfs = &state.vfs.read();
                let file_id = vfs.0.file_id(&path).unwrap();
                std::str::from_utf8(vfs.0.file_contents(file_id))
                    .unwrap()
                    .into()
            },
            params.content_changes,
        );
        state
            .vfs
            .write()
            .0
            .set_file_contents(path, Some(text.into_bytes()));
    }
    Ok(())
}

pub(crate) fn handle_did_open_text_document(
    state: &mut GlobalState,
    params: DidOpenTextDocumentParams,
) -> anyhow::Result<()> {
    log::debug!(
        "Handling did open text document {}",
        params.text_document.uri
    );
    if let Ok(path) = from_proto::vfs_path(&params.text_document.uri) {
        let already_exists = state
            .mem_docs
            .insert(
                path.clone(),
                DocumentData::new(params.text_document.version),
            )
            .is_err();
        if already_exists {
            log::error!("duplicate DidOpenTextDocument: {}", path);
        }
        state
            .vfs
            .write()
            .0
            .set_file_contents(path, Some(params.text_document.text.into_bytes()));

        log::debug!("Handled did open text document",);
    }
    Ok(())
}

pub(crate) fn handle_did_close_text_document(
    state: &mut GlobalState,
    params: DidCloseTextDocumentParams,
) -> anyhow::Result<()> {
    log::debug!(
        "Handling did change text document {}",
        params.text_document.uri
    );

    if let Ok(path) = from_proto::vfs_path(&params.text_document.uri) {
        if state.mem_docs.remove(&path).is_err() {
            tracing::error!("orphan DidCloseTextDocument: {}", path);
        }

        if let Some(file_id) = state.vfs.read().0.file_id(&path) {
            state.diagnostics.clear_native_for(file_id);
        }

        state
            .semantic_tokens_cache
            .lock()
            .remove(&params.text_document.uri);

        if let Some(path) = path.as_path() {
            state.loader.handle.invalidate(path.to_path_buf());
        }
    }
    Ok(())
}

pub(crate) fn handle_did_save_text_document(
    state: &mut GlobalState,
    params: DidSaveTextDocumentParams,
) -> anyhow::Result<()> {
    if let Ok(vfs_path) = from_proto::vfs_path(&params.text_document.uri) {
        if !state.config.compiler_on_save() || run_flycheck(state, vfs_path) {
            return Ok(());
        }
    } else if state.config.compiler_on_save() {
        // No specific flycheck was triggered, so let's trigger all of them.
        for flycheck in state.flycheck.values() {
            flycheck.restart();
        }
    }
    Ok(())
}

pub(crate) fn handle_did_change_watched_files(
    state: &mut GlobalState,
    params: DidChangeWatchedFilesParams,
) -> anyhow::Result<()> {
    for change in params.changes {
        if let Ok(path) = from_proto::abs_path(&change.uri) {
            state.loader.handle.invalidate(path);
        }
    }
    Ok(())
}

pub(crate) fn handle_work_done_progress_cancel(
    state: &mut GlobalState,
    params: WorkDoneProgressCancelParams,
) -> anyhow::Result<()> {
    if let lsp_types::NumberOrString::String(s) = &params.token {
        if let Some(id) = s.strip_prefix("sourcepawn-studio/flycheck/") {
            if let Ok(id) = id.parse::<FileId>() {
                if let Some(flycheck) = state.flycheck.get(&id) {
                    flycheck.cancel();
                }
            }
        }
    }

    // Just ignore this. It is OK to continue sending progress
    // notifications for this token, as the client can't know when
    // we accepted notification.
    Ok(())
}

pub(crate) fn handle_did_change_configuration(
    state: &mut GlobalState,
    _params: DidChangeConfigurationParams,
) -> anyhow::Result<()> {
    log::debug!("Handling did change configuration",);

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
                        this.update_configuration(config, false);
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

fn run_flycheck(state: &mut GlobalState, vfs_path: VfsPath) -> bool {
    let file_id = state.vfs.read().0.file_id(&vfs_path);
    let Some(file_id) = file_id else {
        return false;
    };
    let world = state.snapshot();
    let mut updated = false;
    let task = move || -> std::result::Result<(), Cancelled> {
        // Trigger flychecks for all workspaces that depend on the saved file
        // Crates containing or depending on the saved file
        let root_ids = world
            .analysis
            .projects_for_file(file_id)?
            .into_iter()
            .sorted()
            .unique()
            .collect_vec();

        // Find and trigger corresponding flychecks
        for flycheck in root_ids
            .iter()
            .flat_map(|root_id| world.flycheck.get(root_id))
        {
            updated = true;
            flycheck.restart();
        }

        // No specific flycheck was triggered, so let's trigger all of them.
        if !updated {
            for flycheck in world.flycheck.values() {
                flycheck.restart();
            }
        }
        Ok(())
    };
    state
        .task_pool
        .handle
        .spawn_with_sender(stdx::thread::ThreadIntent::Worker, move |_| {
            // FIXME: The `AssertUnwindSafe` is a workaround. Not sure why this causes problems.
            if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(task)) {
                tracing::error!("flycheck task panicked: {e:?}")
            }
        });
    true
}
