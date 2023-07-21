use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;
use lsp_types::{CompletionContext, Position};
use regex::Regex;

use crate::spitem::SPItem;

use super::matchtoken::MatchToken;

/// Given a prefix line of a document, return if the end of the prefix line is right after a method call
/// i.e after a `.` or `::`.
///
/// # Arguments
///
/// * `pre_line` - Prefix line to check against.
pub(super) fn is_method_call(pre_line: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?:\.|::)\w*$").unwrap();
    }
    RE.is_match(pre_line)
}

/// Given a prefix line of a document, return whether or not the end of the prefix line is right after
/// a constructor call i.e after a `new`.
///
/// # Arguments
///
/// * `pre_line` - Prefix line to check against.
pub(crate) fn is_ctor_call(pre_line: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"new\s+\w*$").unwrap();
    }
    RE.is_match(pre_line)
}

/// Given a prefix line of a document, return whether or not the end of the prefix line is right after
/// a constructor call i.e after a `new`.
///
/// # Arguments
///
/// * `pre_line` - Prefix line to check against.
pub(crate) fn is_doc_completion(
    pre_line: &str,
    position: &Position,
    all_items: &[Arc<RwLock<SPItem>>],
) -> Option<Arc<RwLock<SPItem>>> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"/\*\*?$").unwrap();
    }
    if !RE.is_match(pre_line) {
        return None;
    }
    all_items
        .iter()
        .find(|&item| {
            let item = &*item.read().unwrap();
            if let SPItem::Function(function_item) = item {
                if position.line == function_item.full_range.start.line - 1 {
                    return true;
                }
            }
            false
        })
        .cloned()
}

/// Check if the trigger character of a [Completion request](lsp_types::request::Completion) is a "$".
///
/// # Arguments
///
/// * `context` - [CompletionContext] of the original request.
pub(super) fn is_callback_completion_request(context: Option<CompletionContext>) -> bool {
    if let Some(context) = context {
        if let Some(trigger_character) = context.trigger_character {
            return trigger_character == "$";
        }
    }

    false
}

/// Given a line of a document, return all the words before a given [position](lsp_types::Position).
///
/// # Example
/// ```sourcepawn
/// if(IsValidClient(client))
/// ```
/// will return {`if`, `IsValidClient`} if the cursor is before `client`.
///
/// # Arguments
///
/// * `sub_line` - Sub line to evaluate.
/// * `pos` - Position of the cursor.
pub(super) fn get_line_words(sub_line: &str, pos: Position) -> Vec<Option<MatchToken>> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\w+").unwrap();
    }

    let mut res: Vec<Option<MatchToken>> = vec![];
    for caps in RE.captures_iter(sub_line) {
        res.push(caps.get(0).map(|m| MatchToken {
            _text: m.as_str().to_string(),
            range: lsp_types::Range {
                start: Position {
                    line: pos.line,
                    character: m.start() as u32,
                },
                end: Position {
                    line: pos.line,
                    character: m.end() as u32,
                },
            },
        }));
    }

    res
}
