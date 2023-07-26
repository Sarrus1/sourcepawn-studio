use lsp_types::{CompletionItem, CompletionItemKind};

const DEFAULT_CONSTANT_ITEMS: &[&str] = &["true", "false", "null"];

const DEFAULT_KEYWORD_ITEMS: &[&str] = &[
    "any", "bool", "break", "case", "char", "continue", "float", "Float", "forward", "int",
    "native", "public", "return", "sizeof", "stock", "String", "switch", "view_as", "void",
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

pub(super) fn get_default_completions() -> Vec<CompletionItem> {
    let mut res = vec![];
    for label in DEFAULT_CONSTANT_ITEMS {
        res.push(CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::CONSTANT),
            ..Default::default()
        })
    }

    for label in DEFAULT_KEYWORD_ITEMS {
        res.push(CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        })
    }

    for label in HARDCODED_DEFINES {
        res.push(CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::CONSTANT),
            detail: Some("Hardcoded constant".to_string()),
            ..Default::default()
        })
    }

    res
}
