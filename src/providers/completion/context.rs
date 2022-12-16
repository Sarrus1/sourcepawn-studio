use lazy_static::lazy_static;
use lsp_types::Position;
use regex::Regex;

use super::matchtoken::MatchToken;

/// Given a line of a document, return if the [position](lsp_types::Position) is right after
/// a `.` or a `::`, indicating a methodcall.
///
/// # Arguments
///
/// * `line` - Line to check against.
/// * `pos` - Position of the cursor.
pub(super) fn is_method_call(line: &str, pos: Position) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?:\.|::)\w*$").unwrap();
    }
    let sub_line: String = line.chars().take(pos.character as usize).collect();
    RE.is_match(&sub_line)
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
/// * `line` - Line to evaluate.
/// * `pos` - Position of the cursor.
pub(super) fn get_line_words(line: &str, pos: Position) -> Vec<Option<MatchToken>> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\w+").unwrap();
    }
    let sub_line: String = line.chars().take(pos.character as usize).collect();

    let mut res: Vec<Option<MatchToken>> = vec![];
    for caps in RE.captures_iter(&sub_line) {
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
