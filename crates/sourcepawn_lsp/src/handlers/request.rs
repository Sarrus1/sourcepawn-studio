use std::panic::AssertUnwindSafe;

use anyhow::Context;
use base_db::FileRange;
use ide::{HoverAction, HoverGotoTypeData};
use ide_db::SymbolKind;
use itertools::Itertools;
use lsp_types::{
    SemanticTokensDeltaParams, SemanticTokensFullDeltaResult, SemanticTokensParams,
    SemanticTokensRangeParams, SemanticTokensRangeResult, SemanticTokensResult, Url,
};
use stdx::format_to;
use vfs::FileId;

use crate::{
    global_state::GlobalStateSnapshot,
    lsp::ext::{
        AnalyzerStatusParams, ItemTreeParams, PreprocessedDocumentParams, ProjectMainPathParams,
        ProjectsGraphvizParams, SyntaxTreeParams,
    },
    lsp::{self, from_proto, to_proto},
};

pub(crate) fn handle_completion(
    snap: GlobalStateSnapshot,
    params: lsp_types::CompletionParams,
) -> anyhow::Result<Option<lsp_types::CompletionResponse>> {
    let position = from_proto::file_position(&snap, params.text_document_position.clone())?;
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
    )? {
        return Ok(Some(lsp_types::CompletionResponse::Array(
            completions
                .into_iter()
                .map(|item| {
                    let kind = item.kind;
                    let mut c_item = to_proto::completion_item(&snap, item);
                    match kind {
                        SymbolKind::Local | SymbolKind::Global => {
                            c_item.sort_text = Some("0".to_string())
                        }
                        SymbolKind::Function => c_item.sort_text = Some("0.1".to_string()),
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

pub(crate) fn handle_hover(
    snap: GlobalStateSnapshot,
    params: lsp_types::HoverParams,
) -> anyhow::Result<Option<lsp::ext::Hover>> {
    let pos = from_proto::file_position(&snap, params.text_document_position_params.clone())?;

    let file_id_to_url = &|id: FileId| {
        snap.file_id_to_url(id)
            .to_file_path()
            .ok()
            .and_then(|it| it.to_str().map(|it| it.to_string()))
    };
    let file_id_to_url: AssertUnwindSafe<&dyn Fn(FileId) -> Option<String>> =
        AssertUnwindSafe(file_id_to_url);

    let info = match snap
        .analysis
        .hover(pos, &snap.config.hover(), file_id_to_url)?
    {
        None => return Ok(None),
        Some(it) => it,
    };

    let res = lsp::ext::Hover {
        hover: lsp_types::Hover {
            contents: lsp_types::HoverContents::Markup(to_proto::markup_content(
                info.info.markup,
                snap.config.hover().format,
            )),
            range: Some(info.range),
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

pub(crate) fn handle_semantic_tokens_full(
    snap: GlobalStateSnapshot,
    params: SemanticTokensParams,
) -> anyhow::Result<Option<SemanticTokensResult>> {
    let file_id = from_proto::file_id(&snap, &params.text_document.uri)?;
    let text = snap.analysis.file_text(file_id)?;

    let highlights = snap.analysis.highlight(file_id)?;
    let semantic_tokens = to_proto::semantic_tokens(&text, highlights);

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
    let text = snap.analysis.file_text(file_id)?;

    let highlights = snap.analysis.highlight(file_id)?;

    let semantic_tokens = to_proto::semantic_tokens(&text, highlights);

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
    let text = snap.analysis.file_text(frange.file_id)?;

    let highlights = snap.analysis.highlight_range(frange)?;
    let semantic_tokens = to_proto::semantic_tokens(&text, highlights);

    Ok(Some(semantic_tokens.into()))
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
