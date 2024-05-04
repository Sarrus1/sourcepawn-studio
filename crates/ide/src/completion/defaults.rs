use std::str::FromStr;

use ide_db::SymbolKind;
use smol_str::SmolStr;

use crate::CompletionItem;

const DEFAULT_LITERAL: &[&str] = &["true", "false", "null"];

const DEFAULT_KEYWORD: &[&str] = &[
    "any", "bool", "char", "float", "Float", "int", "String", "const", "static",
];

const DEFAULT_GLOBAL_KEYWORDS: &[&str] = &["stock", "public", "forward", "native", "void"];

const DEFAULT_LOCAL_KEYWORDS: &[&str] = &[
    "continue", "break", "return", "sizeof", "switch", "case", "view_as", "this",
];

const HARDCODED_DEFINES: &[&str] = &[
    "INVALID_FUNCTION",
    "__DATE__",
    "__TIME__",
    "__BINARY_PATH__",
    "__BINARY_NAME__",
    "cellmin",
    "cellmax",
    "EOS",
    "__Pawn",
    "__LINE__",
];

// FIXME: Return an iterator here instead.
pub(super) fn get_default_completions(locals: bool) -> Vec<CompletionItem> {
    let mut res = vec![];
    res.extend(DEFAULT_LITERAL.iter().filter_map(|label| {
        CompletionItem {
            label: SmolStr::from_str(label).ok()?,
            kind: SymbolKind::Literal,
            insert_text: None,
            detail: None,
            documentation: None,
            deprecated: false,
            trigger_call_info: false,
        }
        .into()
    }));

    res.extend(
        DEFAULT_KEYWORD
            .iter()
            .chain(if locals {
                DEFAULT_LOCAL_KEYWORDS
            } else {
                DEFAULT_GLOBAL_KEYWORDS
            })
            .filter_map(|label| {
                CompletionItem {
                    label: SmolStr::from_str(label).ok()?,
                    kind: SymbolKind::Keyword,
                    insert_text: None,
                    detail: None,
                    documentation: None,
                    deprecated: false,
                    trigger_call_info: false,
                }
                .into()
            }),
    );

    res.extend(HARDCODED_DEFINES.iter().filter_map(|label| {
        CompletionItem {
            label: SmolStr::from_str(label).ok()?,
            kind: SymbolKind::Macro,
            insert_text: None,
            detail: Some("Hardcoded constant".to_string()),
            documentation: None,
            deprecated: false,
            trigger_call_info: false,
        }
        .into()
    }));

    res
}
