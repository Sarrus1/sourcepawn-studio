use lsp_types::{Position, Range, TextDocumentContentChangeEvent};

use crate::{line_index::LineIndex, line_index_ext::LineIndexExt};

pub fn apply_document_edit(old_text: &mut String, changes: Vec<TextDocumentContentChangeEvent>) {
    for change in changes {
        let line_index = LineIndex::new(old_text);
        match change.range {
            Some(range) => {
                let range = std::ops::Range::<usize>::from(line_index.offset_lsp_range(range));
                old_text.replace_range(range, &change.text);
            }
            None => {
                *old_text = change.text;
            }
        };
    }
}

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
