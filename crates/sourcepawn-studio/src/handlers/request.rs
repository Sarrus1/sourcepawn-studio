use std::panic::AssertUnwindSafe;

use anyhow::{bail, Context};
use base_db::FileRange;
use ide::{CompletionKind, HoverAction, HoverGotoTypeData};
use ide_db::SymbolKind;
use lsp_types::{
    CallHierarchyIncomingCall, CallHierarchyIncomingCallsParams, CallHierarchyItem,
    CallHierarchyOutgoingCall, CallHierarchyOutgoingCallsParams, CallHierarchyPrepareParams,
    DocumentSymbolResponse, SemanticTokensDeltaParams, SemanticTokensFullDeltaResult,
    SemanticTokensParams, SemanticTokensRangeParams, SemanticTokensRangeResult,
    SemanticTokensResult, SignatureHelp, SignatureHelpParams, Url,
};
use stdx::format_to;
use vfs::FileId;

use crate::{
    global_state::GlobalStateSnapshot,
    lsp::{
        self,
        ext::{
            AnalyzerStatusParams, ItemTreeParams, PreprocessedDocumentParams,
            ProjectMainPathParams, ProjectsGraphvizParams, SyntaxTreeParams,
        },
        from_proto, to_proto,
    },
};

pub(crate) fn handle_resolve_completion(
    snap: GlobalStateSnapshot,
    params: lsp_types::CompletionItem,
) -> anyhow::Result<lsp_types::CompletionItem> {
    let mut item = params;

    let Some(data) = item.data.take() else {
        return Ok(item);
    };
    let Some(new_item) = snap.analysis.resolve_completion(data, item.clone())? else {
        return Ok(item);
    };

    item = new_item;

    Ok(item)
}

pub(crate) fn handle_completion(
    snap: GlobalStateSnapshot,
    params: lsp_types::CompletionParams,
) -> anyhow::Result<Option<lsp_types::CompletionResponse>> {
    let position = from_proto::file_position(&snap, params.text_document_position.clone())?;
    let line_index = snap.file_line_index(position.file_id)?;
    let trigger_character = params
        .context
        .and_then(|it| it.trigger_character.and_then(|it| it.chars().next()));

    let file_id_to_url = &|id: FileId| snap.file_id_to_url(id);
    let file_id_to_url: AssertUnwindSafe<&dyn Fn(FileId) -> Url> = AssertUnwindSafe(file_id_to_url);

    let include_directories = snap.config.include_directories();

    if let Some(completions) = snap.analysis.completions(
        position,
        trigger_character,
        include_directories,
        file_id_to_url,
        snap.config.events_game_name(),
    )? {
        return Ok(Some(lsp_types::CompletionResponse::Array(
            completions
                .into_iter()
                .map(|item| {
                    let kind = item.kind;
                    let mut c_item = to_proto::completion_item(&line_index, item);
                    match kind {
                        CompletionKind::SymbolKind(SymbolKind::Local)
                        | CompletionKind::SymbolKind(SymbolKind::Global) => {
                            c_item.sort_text = Some("0".to_string())
                        }
                        CompletionKind::SymbolKind(SymbolKind::Function) => {
                            c_item.sort_text = Some("0.1".to_string())
                        }
                        _ => (),
                    }
                    c_item
                })
                .collect(),
        )));
    }

    Ok(None)
}

pub(crate) fn handle_goto_definition(
    snap: GlobalStateSnapshot,
    params: lsp_types::GotoDefinitionParams,
) -> anyhow::Result<Option<lsp_types::GotoDefinitionResponse>> {
    log::debug!("goto def: {:?}", params);
    let pos = from_proto::file_position(&snap, params.text_document_position_params.clone())?;

    let targets = match snap.analysis.goto_definition(pos)? {
        None => return Ok(None),
        Some(it) => it,
    };
    let src = FileRange {
        file_id: pos.file_id,
        range: targets.range,
    };

    Ok(Some(to_proto::goto_definition_response(
        &snap,
        Some(src),
        targets.info,
    )?))
}

pub(crate) fn handle_references(
    snap: GlobalStateSnapshot,
    params: lsp_types::ReferenceParams,
) -> anyhow::Result<Option<Vec<lsp_types::Location>>> {
    let pos = from_proto::file_position(&snap, params.text_document_position.clone())?;

    let franges = match snap.analysis.references(pos)? {
        None => return Ok(None),
        Some(it) => it,
    };

    Ok(Some(to_proto::references_response(&snap, franges)?))
}

pub(crate) fn handle_rename(
    snap: GlobalStateSnapshot,
    params: lsp_types::RenameParams,
) -> anyhow::Result<Option<lsp_types::WorkspaceEdit>> {
    let pos = from_proto::file_position(&snap, params.text_document_position.clone())?;

    let source_change = match snap.analysis.rename(pos, &params.new_name)? {
        None => return Ok(None),
        Some(it) => it,
    };

    Ok(Some(to_proto::workspace_edit(&snap, source_change)))
}

pub(crate) fn handle_symbol(
    snap: GlobalStateSnapshot,
    params: lsp_types::DocumentSymbolParams,
) -> anyhow::Result<Option<DocumentSymbolResponse>> {
    let file_id = from_proto::file_id(&snap, &params.text_document.uri)?;
    let line_index = snap.file_line_index(file_id)?;

    let symbols = match snap.analysis.symbols(file_id)? {
        None => return Ok(None),
        Some(it) => it,
    };

    Ok(Some(DocumentSymbolResponse::Nested(
        to_proto::document_symbols(&snap, &line_index, symbols),
    )))
}

pub(crate) fn handle_hover(
    snap: GlobalStateSnapshot,
    params: lsp_types::HoverParams,
) -> anyhow::Result<Option<lsp::ext::Hover>> {
    let pos = from_proto::file_position(&snap, params.text_document_position_params.clone())?;
    let line_index = snap.file_line_index(pos.file_id)?;

    let file_id_to_url = &|id: FileId| {
        snap.file_id_to_url(id)
            .to_file_path()
            .ok()
            .and_then(|it| it.to_str().map(|it| it.to_string()))
    };
    let file_id_to_url: AssertUnwindSafe<&dyn Fn(FileId) -> Option<String>> =
        AssertUnwindSafe(file_id_to_url);

    let info = match snap.analysis.hover(
        pos,
        &snap.config.hover(),
        file_id_to_url,
        snap.config.events_game_name(),
    )? {
        None => return Ok(None),
        Some(it) => it,
    };

    let res = lsp::ext::Hover {
        hover: lsp_types::Hover {
            contents: lsp_types::HoverContents::Markup(to_proto::markup_content(
                info.info.markup,
                snap.config.hover().format,
            )),
            range: line_index.try_range(info.range),
        },
        actions: if snap.config.hover_actions().none() {
            Vec::new()
        } else {
            prepare_hover_actions(&snap, &info.info.actions)
        },
    };

    Ok(res.into())
}

fn goto_type_action_links(
    snap: &GlobalStateSnapshot,
    nav_targets: &[HoverGotoTypeData],
) -> Option<lsp::ext::CommandLinkGroup> {
    if !snap.config.hover_actions().goto_type_def
        || nav_targets.is_empty()
        || !snap.config.client_commands().goto_location
    {
        return None;
    }

    Some(lsp::ext::CommandLinkGroup {
        title: Some("Go to ".into()),
        commands: nav_targets
            .iter()
            .filter_map(|it| {
                to_proto::command::goto_location(snap, &it.nav)
                    .map(|cmd| to_command_link(cmd, it.mod_path.clone()))
            })
            .collect(),
    })
}

fn to_command_link(command: lsp_types::Command, tooltip: String) -> lsp::ext::CommandLink {
    lsp::ext::CommandLink {
        tooltip: Some(tooltip),
        command,
    }
}

fn prepare_hover_actions(
    snap: &GlobalStateSnapshot,
    actions: &[HoverAction],
) -> Vec<lsp::ext::CommandLinkGroup> {
    actions
        .iter()
        .filter_map(|it| match it {
            HoverAction::Implementation(_) => todo!(),
            HoverAction::Reference(_) => todo!(),
            HoverAction::GoToType(targets) => goto_type_action_links(snap, targets),
        })
        .collect()
}

pub(crate) fn handle_signature_help(
    snap: GlobalStateSnapshot,
    params: SignatureHelpParams,
) -> anyhow::Result<Option<SignatureHelp>> {
    let position = from_proto::file_position(&snap, params.text_document_position_params.clone())?;
    let signature = match snap.analysis.signature_help(position)? {
        None => return Ok(None),
        Some(it) => it,
    };

    Ok(Some(to_proto::signature_help(signature)))
}

pub(crate) fn handle_semantic_tokens_full(
    snap: GlobalStateSnapshot,
    params: SemanticTokensParams,
) -> anyhow::Result<Option<SemanticTokensResult>> {
    let file_id = from_proto::file_id(&snap, &params.text_document.uri)?;
    let line_index = snap.file_line_index(file_id)?;

    let text = snap.analysis.file_text(file_id)?;

    let highlights = snap.analysis.highlight(file_id)?;
    let semantic_tokens = to_proto::semantic_tokens(&text, &line_index, highlights);

    // Unconditionally cache the tokens
    snap.semantic_tokens_cache
        .lock()
        .insert(params.text_document.uri, semantic_tokens.clone());

    Ok(Some(semantic_tokens.into()))
}

pub(crate) fn handle_semantic_tokens_full_delta(
    snap: GlobalStateSnapshot,
    params: SemanticTokensDeltaParams,
) -> anyhow::Result<Option<SemanticTokensFullDeltaResult>> {
    let file_id = from_proto::file_id(&snap, &params.text_document.uri)?;
    let line_index = snap.file_line_index(file_id)?;
    let text = snap.analysis.file_text(file_id)?;

    let highlights = snap.analysis.highlight(file_id)?;

    let semantic_tokens = to_proto::semantic_tokens(&text, &line_index, highlights);

    let cached_tokens = snap
        .semantic_tokens_cache
        .lock()
        .remove(&params.text_document.uri);

    if let Some(
        cached_tokens @ lsp_types::SemanticTokens {
            result_id: Some(prev_id),
            ..
        },
    ) = &cached_tokens
    {
        if *prev_id == params.previous_result_id {
            let delta = to_proto::semantic_token_delta(cached_tokens, &semantic_tokens);
            snap.semantic_tokens_cache
                .lock()
                .insert(params.text_document.uri, semantic_tokens);
            return Ok(Some(delta.into()));
        }
    }

    // Clone first to keep the lock short
    let semantic_tokens_clone = semantic_tokens.clone();
    snap.semantic_tokens_cache
        .lock()
        .insert(params.text_document.uri, semantic_tokens_clone);

    Ok(Some(semantic_tokens.into()))
}

pub(crate) fn handle_semantic_tokens_range(
    snap: GlobalStateSnapshot,
    params: SemanticTokensRangeParams,
) -> anyhow::Result<Option<SemanticTokensRangeResult>> {
    let frange = from_proto::file_range(&snap, &params.text_document, params.range)?;
    let line_index = snap.file_line_index(frange.file_id)?;
    let text = snap.analysis.file_text(frange.file_id)?;

    let highlights = snap.analysis.highlight_range(frange)?;
    let semantic_tokens = to_proto::semantic_tokens(&text, &line_index, highlights);

    Ok(Some(semantic_tokens.into()))
}

pub(crate) fn handle_call_hierarchy_prepare(
    snap: GlobalStateSnapshot,
    params: CallHierarchyPrepareParams,
) -> anyhow::Result<Option<Vec<CallHierarchyItem>>> {
    let fpos = from_proto::file_position(&snap, params.text_document_position_params)?;

    let call_items = match snap.analysis.call_hierarchy_prepare(fpos)? {
        Some(it) => it,
        None => return Ok(None),
    };

    Ok(Some(to_proto::call_hierarchy_items(&snap, call_items)))
}

pub(crate) fn handle_call_hierarchy_incoming(
    snap: GlobalStateSnapshot,
    params: CallHierarchyIncomingCallsParams,
) -> anyhow::Result<Option<Vec<CallHierarchyIncomingCall>>> {
    let mut item = params.item;

    let Some(data) = item.data.take() else {
        bail!("no data attached to the incoming item");
    };
    let Some(incoming_items) = snap.analysis.call_hierarchy_incoming(data)? else {
        bail!("could not resolve incoming calls");
    };

    Ok(Some(to_proto::call_hierarchy_incoming(
        &snap,
        incoming_items,
    )))
}

pub(crate) fn handle_call_hierarchy_outgoing(
    snap: GlobalStateSnapshot,
    params: CallHierarchyOutgoingCallsParams,
) -> anyhow::Result<Option<Vec<CallHierarchyOutgoingCall>>> {
    let mut item = params.item;

    let Some(data) = item.data.take() else {
        bail!("no data attached to the incoming item");
    };
    let Some(outgoing_items) = snap.analysis.call_hierarchy_outgoing(data)? else {
        bail!("could not resolve incoming calls");
    };

    Ok(Some(to_proto::call_hierarchy_outgoing(
        &snap,
        outgoing_items,
    )))
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

pub(crate) fn handle_projects_graphviz(
    snap: GlobalStateSnapshot,
    _params: ProjectsGraphvizParams,
) -> anyhow::Result<String> {
    let graph = snap.analysis.graph()?;

    graph
        .to_graphviz(|id| {
            let path = snap.vfs_read().file_path(id);
            path.name_and_extension()
                .map(|(name, ext)| format!("{}.{}", name, ext.unwrap_or_default()))
        })
        .ok_or_else(|| anyhow::anyhow!("Failed to generate graphviz"))
}

pub(crate) fn handle_preprocessed_document(
    snap: GlobalStateSnapshot,
    params: PreprocessedDocumentParams,
) -> anyhow::Result<String> {
    let uri = params
        .text_document
        .ok_or_else(|| anyhow::anyhow!("No uri received in request"))?
        .uri;
    let file_id = from_proto::file_id(&snap, &uri)?;

    snap.analysis
        .preprocessed_text(file_id)
        .context("Failed to preprocess document")
        .map(|it| it.to_string())
}

pub(crate) fn handle_item_tree(
    snap: GlobalStateSnapshot,
    params: ItemTreeParams,
) -> anyhow::Result<String> {
    let uri = params
        .text_document
        .ok_or_else(|| anyhow::anyhow!("No uri received in request"))?
        .uri;
    let file_id = from_proto::file_id(&snap, &uri)?;

    snap.analysis
        .pretty_item_tree(file_id)
        .context("Failed to get the item tree")
}

pub(crate) fn handle_analyzer_status(
    snap: GlobalStateSnapshot,
    params: AnalyzerStatusParams,
) -> anyhow::Result<String> {
    let mut buf = String::new();

    let mut file_id = None;
    if let Some(tdi) = params.text_document {
        match from_proto::file_id(&snap, &tdi.uri) {
            Ok(it) => file_id = Some(it),
            Err(_) => format_to!(buf, "file {} not found in vfs", tdi.uri),
        }
    }

    // if snap.workspaces.is_empty() {
    //     buf.push_str("No workspaces\n")
    // } else {
    //     buf.push_str("Workspaces:\n");
    //     format_to!(
    //         buf,
    //         "Loaded {:?} packages across {} workspace{}.\n",
    //         snap.workspaces
    //             .iter()
    //             .map(|w| w.n_packages())
    //             .sum::<usize>(),
    //         snap.workspaces.len(),
    //         if snap.workspaces.len() == 1 { "" } else { "s" }
    //     );

    //     format_to!(
    //         buf,
    //         "Workspace root folders: {:?}",
    //         snap.workspaces
    //             .iter()
    //             .flat_map(|ws| ws.workspace_definition_path())
    //             .collect::<Vec<&AbsPath>>()
    //     );
    // }
    format_to!(
        buf,
        "\nVfs memory usage: {}\n",
        profile::Bytes::new(snap.vfs_memory_usage() as _)
    );
    buf.push_str("\nAnalysis:\n");
    buf.push_str(
        &snap
            .analysis
            .status(file_id)
            .unwrap_or_else(|_| "Analysis retrieval was cancelled".to_owned()),
    );
    Ok(buf)
}

pub(crate) fn handle_project_main_path(
    snap: GlobalStateSnapshot,
    params: ProjectMainPathParams,
) -> anyhow::Result<Url> {
    let uri = params
        .uri
        .ok_or_else(|| anyhow::anyhow!("No uri received in request"))?;
    let file_id = from_proto::file_id(&snap, &uri)?;

    snap.analysis
        .projects_for_file(file_id)
        .context("Failed to get project for file")?
        .first()
        .map(|it| to_proto::url(&snap, *it))
        .ok_or_else(|| anyhow::anyhow!("No project found for file"))
}
