use fxhash::FxHashMap;
use syntax::range_contains_pos;

use crate::{ArgsMap, Offset};

/// Convert a position seen by the user to a position seen by the server (preprocessed).
///
/// Will try to look for a mapped range of a macro argument and return the offsetted position.
/// If no mapped range is found, will try to apply the offsets to the position.
///
/// # Arguments
///
/// * `args_map` - The preprocessed arguments map.
/// * `offsets` - The preprocessed offsets.
/// * `pos` - The position to convert.
///
/// # Returns
///
/// The source position range, if a mapped range was found.
pub fn u_pos_to_s_pos(
    args_map: &ArgsMap,
    offsets: &FxHashMap<u32, Vec<Offset>>,
    pos: &mut lsp_types::Position,
) -> Option<lsp_types::Range> {
    let mut source_u_range = None;

    match args_map.get(&pos.line).and_then(|args| {
        args.iter()
            .find(|(range, _)| range_contains_pos(range, pos))
    }) {
        Some((u_range, s_range)) => {
            *pos = s_range.start;
            source_u_range = Some(*u_range);
        }
        None => {
            if let Some(diff) = offsets.get(&pos.line).map(|offsets| {
                offsets
                    .iter()
                    .filter(|offset| offset.range.end.character <= pos.character)
                    .map(|offset| offset.diff.saturating_sub_unsigned(offset.args_diff))
                    .sum::<i32>()
            }) {
                *pos = lsp_types::Position {
                    line: pos.line,
                    character: pos.character.saturating_add_signed(diff),
                };
            }
        }
    }

    source_u_range
}

/// Convert a range seen by the server to a range seen by the user.
pub fn s_range_to_u_range(
    offsets: &FxHashMap<u32, Vec<Offset>>,
    mut s_range: lsp_types::Range,
) -> lsp_types::Range {
    if let Some(offsets) = offsets.get(&s_range.start.line) {
        for offset in offsets.iter() {
            if offset.range.start.character < s_range.start.character {
                s_range.start.character = s_range
                    .start
                    .character
                    .saturating_add_signed(-offset.diff.saturating_sub_unsigned(offset.args_diff));
            }
        }
        for offset in offsets.iter() {
            if offset.range.start.character < s_range.end.character {
                s_range.end.character = s_range
                    .end
                    .character
                    .saturating_add_signed(-offset.diff.saturating_sub_unsigned(offset.args_diff));
            }
        }
    }

    s_range
}
