use std::{
    path,
    sync::atomic::{AtomicU32, Ordering},
};

use base_db::FileRange;
use ide::{Cancellable, Highlight, HlMod, HlRange, HlTag, Markup, NavigationTarget, Severity};
use ide_db::SymbolKind;
use itertools::Itertools;
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
    snap: &GlobalStateSnapshot,
    item: ide::CompletionItem,
) -> lsp_types::CompletionItem {
    lsp_types::CompletionItem {
        label: item.label.to_string(),
        kind: Some(completion_item_kind(item.kind)),
        ..Default::default()
    }
}

pub(crate) fn completion_item_kind(kind: SymbolKind) -> lsp_types::CompletionItemKind {
    use lsp_types::CompletionItemKind as CK;
    match kind {
        SymbolKind::Function => CK::FUNCTION,
        SymbolKind::Constructor => CK::CONSTRUCTOR,
        SymbolKind::Destructor => CK::CONSTRUCTOR,
        SymbolKind::Struct => CK::STRUCT,
        SymbolKind::Enum => CK::ENUM,
        SymbolKind::Variant => CK::ENUM_MEMBER,
        SymbolKind::Macro => CK::CONSTANT,
        SymbolKind::Local => CK::VARIABLE,
        SymbolKind::Field => CK::FIELD,
        SymbolKind::Method => CK::METHOD,
        SymbolKind::Typedef => CK::INTERFACE,
        SymbolKind::Typeset => CK::INTERFACE,
        SymbolKind::Functag => CK::INTERFACE,
        SymbolKind::Funcenum => CK::INTERFACE,
        SymbolKind::EnumStruct => CK::STRUCT,
        SymbolKind::Methodmap => CK::CLASS,
        SymbolKind::Property => CK::PROPERTY,
        SymbolKind::Global => CK::VARIABLE,
        SymbolKind::Keyword => CK::KEYWORD,
        SymbolKind::Literal => CK::KEYWORD,
        SymbolKind::Directory => CK::FOLDER,
        SymbolKind::File => CK::FILE,
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
