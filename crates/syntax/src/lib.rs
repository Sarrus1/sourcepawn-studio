use lsp_types::{Position, Range, Url};
use std::sync::Arc;
use vfs::FileId;

mod generated;
mod tests;
pub mod utils;

pub use generated::TSKind;

/// Represents a location inside a resource, such as a line inside a text file.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Reference {
    /// [FileId](FileId) of the location.
    pub file_id: FileId,

    /// [Uri](Url) of the location.
    pub uri: Arc<Url>,

    /// Range of the location.
    pub range: Range,

    /// User visible range of the location.
    pub v_range: Range,
}

/// Returns true if [range](lsp_types::Range) a and [range](lsp_types::Range) b are equal.
///
/// # Arguments
///
/// * `a` - [Range](lsp_types::Range) to check against.
/// * `b` - [Range](lsp_types::Range) to check against.
pub fn range_equals_range(a: &Range, b: &Range) -> bool {
    if a.start.line != b.start.line || a.end.line != b.end.line {
        return false;
    }
    if a.start.character != b.start.character || a.end.character != b.end.character {
        return false;
    }

    true
}

/// Extracts the filename from a [Uri](Url). Returns [None] if it does not exist.
///
/// # Arguments
///
/// * `uri` - [Uri](Url) to extract.
pub fn uri_to_file_name(uri: &Url) -> Option<String> {
    match uri.to_file_path() {
        Ok(path) => match path.as_path().file_name() {
            Some(file_name) => file_name.to_str().map(|file_name| file_name.to_string()),
            None => None,
        },
        Err(_) => None,
    }
}

/// Returns true if a [Position] is contained in a [Range].
///
/// # Arguments
///
/// * `range` - [Range] to check against.
/// * `position` - [Position] to check against.
pub fn range_contains_pos(range: &Range, position: &Position) -> bool {
    if range.start.line > position.line || range.end.line < position.line {
        return false;
    }
    if range.start.line == position.line && range.start.character > position.character {
        return false;
    }
    if range.end.line == position.line && range.end.character < position.character {
        return false;
    }

    true
}
