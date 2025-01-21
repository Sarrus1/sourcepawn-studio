//! This module provides completions/hovers for event names and utilities to
//! check if events completions/hovers should be provided for a given node.

use completion_data::DATABASE;
use ide_db::Documentation;
use preprocessor::SourceMap;
use smol_str::ToSmolStr;
use syntax::{utils::ts_range_to_text_range, TSKind};
use tree_sitter::Node;

use crate::{hover::HoverResult, CompletionItem, CompletionKind, Markup, RangeInfo};

/// Returns the event name if the node is an event name.
///
/// This will check if the node is a string literal, and if it is the first
/// argument of a function call, and if the function's name is either:
/// - `HookEvent`
/// - `HookEventEx`
/// - `UnhookEvent`
///
/// # Arguments
/// - `node`: The node to check
/// - `source`: The preprocessed source code
///
/// # Returns
/// The event name if the node is an event name, otherwise `None`.
pub fn event_name(node: &Node, source: &str) -> Option<String> {
    if TSKind::from(node) != TSKind::string_literal {
        return None;
    }
    let prev_sibling = node.prev_sibling()?;
    if TSKind::from(&prev_sibling) != TSKind::anon_LPAREN {
        // Not element of a function call, cannot be an event name
        return None;
    }
    let parent = node.parent()?;
    if TSKind::from(&parent) != TSKind::call_arguments {
        return None;
    }
    let function = parent.prev_named_sibling()?;
    if TSKind::from(&function) != TSKind::identifier {
        return None;
    }
    let Ok(name) = function.utf8_text(source.as_bytes()) else {
        return None;
    };

    if !matches!(name, "HookEvent" | "HookEventEx" | "UnhookEvent") {
        return None;
    }

    let raw_name = node.utf8_text(source.as_bytes()).ok()?;

    raw_name.trim_matches('"').to_string().into()
}

/// Returns completions for event names.
///
/// If `events_game_name` is `Some`, and if the game exits in the database,
/// only completions for the given game will be returned.
///
/// # Arguments
/// - `events_game_name`: The name of the game to get completions for
pub fn events_completions(events_game_name: Option<&str>) -> Vec<CompletionItem> {
    if let Some(game_name) = events_game_name {
        if let Some(game) = DATABASE.get(game_name) {
            return game
                .events()
                .iter()
                .cloned()
                .chain(DATABASE.generic_events())
                .map(|ev| CompletionItem {
                    label: ev.name().to_smolstr(),
                    kind: CompletionKind::Literal,
                    detail: Some(game_name.to_string()),
                    documentation: Documentation::from(&ev).into(),
                    ..Default::default()
                })
                .collect();
        }
    }
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

/// Returns hover information for an event.
///
/// If `events_game_name` is `Some`, and if the game exits in the database, only return the
/// hover information for the given game. Otherwise, return hover information for all games.
///
/// # Arguments
/// - `events_game_name`: The name of the game to get hover information for
/// - `name`: The name of the event
/// - `node`: The node of the string literal of the event name
/// - `offsets`: The preprocessor offsets
pub fn event_hover(
    events_game_name: Option<&str>,
    name: &str,
    node: &Node,
    source_map: &SourceMap,
) -> Option<RangeInfo<HoverResult>> {
    if let Some(game) = events_game_name {
        if let Some(ev) = DATABASE.get_event(game, name) {
            let markup = Markup::from(format!(
                "## {}\n\n{}",
                game,
                Documentation::from(ev).to_markdown()
            ));
            let res = HoverResult {
                markup,
                actions: Default::default(),
            };
            return Some(RangeInfo::new(
                source_map.closest_u_range_always(ts_range_to_text_range(&node.range())),
                res,
            ));
        }

        None
    } else {
        let mut res = Vec::new();
        DATABASE
            .get_events(name)
            .into_iter()
            .for_each(|(game, ev)| {
                res.push(format!("## {}", game));
                res.push(Documentation::from(&ev).to_markdown());
            });
        Some(RangeInfo::new(
            source_map.closest_u_range_always(ts_range_to_text_range(&node.range())),
            HoverResult {
                markup: Markup::from(res.join("\n\n")),
                actions: Default::default(),
            },
        ))
    }
}
