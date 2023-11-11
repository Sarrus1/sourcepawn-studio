use lsp_types::{Position, Range};

/// Returns the arithmetic average of a [range](lsp_types::Range) as a [position](lsp_types::Position).
///
/// # Arguments
///
/// * `range` - [Range](lsp_types::Range) to average.
pub fn range_to_position_average(range: &Range) -> Position {
    Position {
        line: (range.start.line + range.end.line) / 2,
        character: (range.start.character + range.end.character) / 2,
    }
}
