use std::{
    path,
    sync::atomic::{AtomicU32, Ordering},
};

use base_db::FileRange;
use ide::{
    Cancellable, CompletionKind, Highlight, HlMod, HlRange, HlTag, Markup, NavigationTarget,
    Severity, SignatureHelp,
};
use ide_db::{SourceChange, SymbolKind};
use itertools::Itertools;
use lsp_types::TextEdit;
use paths::AbsPath;
use vfs::FileId;

use crate::global_state::GlobalStateSnapshot;

use super::semantic_tokens;

pub(crate) fn goto_definition_response(
    snap: &GlobalStateSnapshot,
    src: Option<FileRange>,
    targets: Vec<NavigationTarget>,
) -> Cancellable<lsp_types::GotoDefinitionResponse> {
    let links = targets
        .into_iter()
        .map(|nav| location_link(snap, src, nav))
        .collect::<Cancellable<Vec<_>>>()?;
    Ok(links.into())
}

pub(crate) fn location_link(
    snap: &GlobalStateSnapshot,
    src: Option<FileRange>,
    target: NavigationTarget,
) -> Cancellable<lsp_types::LocationLink> {
    let origin_selection_range = src.map(|it| it.range);
    let (target_uri, target_range, target_selection_range) = location_info(snap, target)?;
    let res = lsp_types::LocationLink {
        origin_selection_range,
        target_uri,
        target_range,
        target_selection_range,
    };

    Ok(res)
}

pub(crate) fn references_response(
    snap: &GlobalStateSnapshot,
    targets: Vec<FileRange>,
) -> Cancellable<Vec<lsp_types::Location>> {
    let locations = targets
        .into_iter()
        .map(|frange| location(snap, frange))
        .collect::<Cancellable<Vec<_>>>()?;

    Ok(locations)
}

fn location_info(
    snap: &GlobalStateSnapshot,
    target: NavigationTarget,
) -> Cancellable<(lsp_types::Url, lsp_types::Range, lsp_types::Range)> {
    let target_uri = url(snap, target.file_id);
    let target_range = target.full_range;
    let target_selection_range = target.focus_range.unwrap_or(target_range);
    Ok((target_uri, target_range, target_selection_range))
}

pub(crate) fn markup_content(
    markup: Markup,
    kind: ide::HoverDocFormat,
) -> lsp_types::MarkupContent {
    let kind = match kind {
        ide::HoverDocFormat::Markdown => lsp_types::MarkupKind::Markdown,
        ide::HoverDocFormat::PlainText => lsp_types::MarkupKind::PlainText,
    };
    // let value = format_docs(&Documentation::new(markup.into()));
    let value = markup.to_string();
    lsp_types::MarkupContent { kind, value }
}

pub(crate) fn url(snap: &GlobalStateSnapshot, file_id: FileId) -> lsp_types::Url {
    snap.file_id_to_url(file_id)
}

static TOKEN_RESULT_COUNTER: AtomicU32 = AtomicU32::new(1);

pub(crate) fn semantic_tokens(_text: &str, highlights: Vec<HlRange>) -> lsp_types::SemanticTokens {
    let id = TOKEN_RESULT_COUNTER
        .fetch_add(1, Ordering::SeqCst)
        .to_string();
    let mut builder = semantic_tokens::SemanticTokensBuilder::new(id);

    for highlight_range in highlights {
        if highlight_range.highlight.is_empty() {
            continue;
        }

        let (ty, mods) = semantic_token_type_and_modifiers(highlight_range.highlight);

        let token_index = semantic_tokens::type_index(ty);
        let modifier_bitset = mods.0;
        builder.push(highlight_range.range, token_index, modifier_bitset);
    }

    builder.build()
}

pub(crate) fn semantic_token_delta(
    previous: &lsp_types::SemanticTokens,
    current: &lsp_types::SemanticTokens,
) -> lsp_types::SemanticTokensDelta {
    let result_id = current.result_id.clone();
    let edits = semantic_tokens::diff_tokens(&previous.data, &current.data);
    lsp_types::SemanticTokensDelta { result_id, edits }
}

fn semantic_token_type_and_modifiers(
    highlight: Highlight,
) -> (lsp_types::SemanticTokenType, semantic_tokens::ModifierSet) {
    let mut mods = semantic_tokens::ModifierSet::default();
    let type_ = match highlight.tag {
        HlTag::Symbol(symbol) => match symbol {
            SymbolKind::Macro => semantic_tokens::MACRO,
            _ => todo!(),
        },
        HlTag::None => semantic_tokens::GENERIC,
    };

    for modifier in highlight.mods.iter() {
        let modifier = match modifier {
            HlMod::Macro => semantic_tokens::MACRO_MODIFIER,
        };
        mods |= modifier;
    }

    (type_, mods)
}

pub(crate) fn diagnostic_severity(severity: Severity) -> lsp_types::DiagnosticSeverity {
    match severity {
        Severity::Error => lsp_types::DiagnosticSeverity::ERROR,
        Severity::Warning => lsp_types::DiagnosticSeverity::WARNING,
        Severity::WeakWarning => lsp_types::DiagnosticSeverity::HINT,
    }
}

/// Returns a `Url` object from a given path, will lowercase drive letters if present.
/// This will only happen when processing windows paths.
///
/// When processing non-windows path, this is essentially the same as `Url::from_file_path`.
pub(crate) fn url_from_abs_path(path: &AbsPath) -> lsp_types::Url {
    let url = lsp_types::Url::from_file_path(path).unwrap();
    match path.as_ref().components().next() {
        Some(path::Component::Prefix(prefix))
            if matches!(
                prefix.kind(),
                path::Prefix::Disk(_) | path::Prefix::VerbatimDisk(_)
            ) =>
        {
            // Need to lowercase driver letter
        }
        _ => return url,
    }

    let driver_letter_range = {
        let (scheme, drive_letter, _rest) = match url.as_str().splitn(3, ':').collect_tuple() {
            Some(it) => it,
            None => return url,
        };
        let start = scheme.len() + ':'.len_utf8();
        start..(start + drive_letter.len())
    };

    // Note: lowercasing the `path` itself doesn't help, the `Url::parse`
    // machinery *also* canonicalizes the drive letter. So, just massage the
    // string in place.
    let mut url: String = url.into();
    url[driver_letter_range].make_ascii_lowercase();
    lsp_types::Url::parse(&url).unwrap()
}

pub(crate) fn location(
    snap: &GlobalStateSnapshot,
    frange: FileRange,
) -> Cancellable<lsp_types::Location> {
    let url = url(snap, frange.file_id);
    let loc = lsp_types::Location::new(url, frange.range);
    Ok(loc)
}

pub(crate) fn completion_item(
    _snap: &GlobalStateSnapshot,
    item: ide::CompletionItem,
) -> lsp_types::CompletionItem {
    lsp_types::CompletionItem {
        label: item.label.to_string(),
        insert_text: item.insert_text.map(|it| it.to_string()),
        kind: Some(completion_item_kind(item.kind)),
        insert_text_format: {
            if item.kind == CompletionKind::Snippet {
                Some(lsp_types::InsertTextFormat::SNIPPET)
            } else {
                Some(lsp_types::InsertTextFormat::PLAIN_TEXT)
            }
        },
        filter_text: item.filter_text,
        text_edit: item.text_edit.map(|(range, new_text)| {
            lsp_types::CompletionTextEdit::Edit(TextEdit::new(range, new_text))
        }),
        deprecated: item.deprecated.into(),
        tags: if item.deprecated {
            Some(vec![lsp_types::CompletionItemTag::DEPRECATED])
        } else {
            None
        },
        detail: item.detail.map(|it| it.to_string()),
        documentation: item.documentation.map(Into::into),
        data: item.data.and_then(|it| serde_json::to_value(it).ok()),
        ..Default::default()
    }
}

pub(crate) fn completion_item_kind(kind: CompletionKind) -> lsp_types::CompletionItemKind {
    use lsp_types::CompletionItemKind as CK;

    match kind {
        CompletionKind::SymbolKind(SymbolKind::Function) => CK::FUNCTION,
        CompletionKind::SymbolKind(SymbolKind::Forward) => CK::INTERFACE,
        CompletionKind::SymbolKind(SymbolKind::Constructor) => CK::CONSTRUCTOR,
        CompletionKind::SymbolKind(SymbolKind::Destructor) => CK::CONSTRUCTOR,
        CompletionKind::SymbolKind(SymbolKind::Struct) => CK::STRUCT,
        CompletionKind::SymbolKind(SymbolKind::Enum) => CK::ENUM,
        CompletionKind::SymbolKind(SymbolKind::Variant) => CK::ENUM_MEMBER,
        CompletionKind::SymbolKind(SymbolKind::Macro) => CK::CONSTANT,
        CompletionKind::SymbolKind(SymbolKind::Local) => CK::VARIABLE,
        CompletionKind::SymbolKind(SymbolKind::Field) => CK::FIELD,
        CompletionKind::SymbolKind(SymbolKind::Method) => CK::METHOD,
        CompletionKind::SymbolKind(SymbolKind::Typedef) => CK::INTERFACE,
        CompletionKind::SymbolKind(SymbolKind::Typeset) => CK::INTERFACE,
        CompletionKind::SymbolKind(SymbolKind::Functag) => CK::INTERFACE,
        CompletionKind::SymbolKind(SymbolKind::Funcenum) => CK::INTERFACE,
        CompletionKind::SymbolKind(SymbolKind::EnumStruct) => CK::STRUCT,
        CompletionKind::SymbolKind(SymbolKind::Methodmap) => CK::CLASS,
        CompletionKind::SymbolKind(SymbolKind::Property) => CK::PROPERTY,
        CompletionKind::SymbolKind(SymbolKind::Global) => CK::VARIABLE,
        CompletionKind::Keyword => CK::KEYWORD,
        CompletionKind::Literal => CK::KEYWORD,
        CompletionKind::Directory => CK::FOLDER,
        CompletionKind::File => CK::FILE,
        CompletionKind::Snippet => CK::SNIPPET,
    }
}

pub(crate) fn signature_help(sig: SignatureHelp) -> lsp_types::SignatureHelp {
    lsp_types::SignatureHelp {
        signatures: vec![lsp_types::SignatureInformation {
            label: sig.signature,
            documentation: sig.doc.clone().map(|doc| doc.into()),
            parameters: sig
                .parameters
                .into_iter()
                .map(|it| lsp_types::ParameterInformation {
                    label: lsp_types::ParameterLabel::Simple(it.clone()),
                    documentation: sig
                        .doc
                        .clone()
                        // This is not efficient, but it's not a hot path.
                        .and_then(|doc| doc.param_description(&it).map(|it| it.into())),
                })
                .collect_vec()
                .into(),
            active_parameter: sig.active_parameter,
        }],
        active_signature: Default::default(),
        active_parameter: sig.active_parameter,
    }
}

pub(crate) fn workspace_edit(
    snap: &GlobalStateSnapshot,
    source_change: SourceChange,
) -> lsp_types::WorkspaceEdit {
    let changes = source_change
        .source_file_edits
        .into_iter()
        .map(|(file_id, edits)| {
            let uri = url(snap, file_id);
            let text_edits = edits
                .into_iter()
                .map(|edit| lsp_types::TextEdit::new(edit.range, edit.new_text))
                .collect();
            (uri, text_edits)
        })
        .collect();

    lsp_types::WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    }
}

pub(crate) mod command {
    use base_db::FileRange;
    use ide::NavigationTarget;
    use serde_json::to_value;

    use crate::{global_state::GlobalStateSnapshot, lsp::to_proto::location_link};

    use super::location;

    pub(crate) fn goto_location(
        snap: &GlobalStateSnapshot,
        nav: &NavigationTarget,
    ) -> Option<lsp_types::Command> {
        let value = if snap.config.location_link() {
            let link = location_link(snap, None, nav.clone()).ok()?;
            to_value(link).ok()?
        } else {
            let range = FileRange {
                file_id: nav.file_id,
                range: nav.focus_or_full_range(),
            };
            let location = location(snap, range).ok()?;
            to_value(location).ok()?
        };

        Some(lsp_types::Command {
            title: nav.name.to_string(),
            command: "sourcepawn-vscode.gotoLocation".into(),
            arguments: Some(vec![value]),
        })
    }
}
