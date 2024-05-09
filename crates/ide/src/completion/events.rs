//! This module provides completions for event names and utilities to
//! check if events completions should be provided for a given node.

use completion_data::DATABASE;
use smol_str::ToSmolStr;
use syntax::TSKind;
use tree_sitter::Node;

use crate::{CompletionItem, CompletionKind};

/// Returns whether the given node is an event name.
///
/// This will check if the node is a string literal, and if it is the first
/// argument of a function call, and if the function's name is either:
/// - `HookEvent`
/// - `HookEventEx`
/// - `UnhookEvent`
///
/// # Parameters
/// - `node`: The node to check
/// - `source`: The preprocessed source code
pub fn is_event_name(node: &Node, source: &str) -> bool {
    if TSKind::from(node) != TSKind::string_literal {
        return false;
    }
    let Some(prev_sibling) = node.prev_sibling() else {
        return false;
    };
    if TSKind::from(&prev_sibling) != TSKind::anon_LPAREN {
        // Not element of a function call, cannot be an event name
        return false;
    }
    let Some(parent) = node.parent() else {
        return false;
    };
    if TSKind::from(&parent) != TSKind::call_arguments {
        return false;
    }
    let Some(function) = parent.prev_named_sibling() else {
        return false;
    };
    if TSKind::from(&function) != TSKind::identifier {
        return false;
    }
    let Ok(name) = function.utf8_text(source.as_bytes()) else {
        return false;
    };

    matches!(name, "HookEvent" | "HookEventEx" | "UnhookEvent")
}

/// Returns completions for event names.
pub fn events_completions() -> Vec<CompletionItem> {
    DATABASE
        .iter()
        .flat_map(|(_, game)| {
            game.events().iter().map(|ev| CompletionItem {
                label: ev.name().to_smolstr(),
                kind: CompletionKind::Literal,
                detail: Some(game.name().to_string()),
                ..Default::default()
            })
        })
        .collect()
}
