use std::{fs::File, io::Read, path::PathBuf};

use lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};

use crate::{line_index::LineIndex, line_index_ext::LineIndexExt};

/// Convert a Tree-sitter [Range](tree_sitter::Range) to an LSP [Range](lsp_types::Range).
///
/// # Arguments
///
/// * `range` - Tree-sitter [Range](tree_sitter::Range) to convert.
pub fn ts_range_to_lsp_range(range: &tree_sitter::Range) -> lsp_types::Range {
    let start = point_to_lsp_position(&range.start_point);
    let end = point_to_lsp_position(&range.end_point);
    lsp_types::Range { start, end }
}

/// Convert a Tree-sitter [Point](tree_sitter::Point) to an LSP [Position](lsp_types::Position).
///
/// # Arguments
///
/// * `point` - Tree-sitter [Point](tree_sitter::Point) to convert.
pub fn point_to_lsp_position(point: &tree_sitter::Point) -> lsp_types::Position {
    Position::new(point.row as u32, point.column as u32)
}

/// Add `.inc` to a trimmed include text if it does not have an extension (.sp or .inc).
///
/// # Arguments
///
/// * `include_text` - Include text to edit.
pub fn add_include_extension(include_text: &mut String, amxxpawn_mode: bool) -> &String {
    if amxxpawn_mode {
        if !include_text.ends_with(".sma") && !include_text.ends_with(".inc") {
            include_text.push_str(".inc");
        }
    } else if !include_text.ends_with(".sp") && !include_text.ends_with(".inc") {
        include_text.push_str(".inc");
    }

    include_text
}

pub fn normalize_uri(uri: &mut lsp_types::Url) {
    fn fix_drive_letter(text: &str) -> Option<String> {
        if !text.is_ascii() {
            return None;
        }

        match &text[1..] {
            ":" => Some(text.to_ascii_uppercase()),
            "%3A" | "%3a" => Some(format!("{}:", text[0..1].to_ascii_uppercase())),
            _ => None,
        }
    }

    if let Some(mut segments) = uri.path_segments() {
        if let Some(mut path) = segments.next().and_then(fix_drive_letter) {
            for segment in segments {
                path.push('/');
                path.push_str(segment);
            }

            uri.set_path(&path);
        }
    }

    uri.set_fragment(None);
}

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

/// Returns true if a [Position] is contained in a [Range].
///
/// # Arguments
///
/// * `range` - [Range] to check against.
/// * `position` - [Position] to check against.
pub fn range_contains_pos(range: Range, position: Position) -> bool {
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

/// Returns true if [Range] a contains [Range] b.
///
/// # Arguments
///
/// * `a` - [Range] to check against.
/// * `b` - [Range] to check against.
pub fn range_contains_range(a: &Range, b: &Range) -> bool {
    if b.start.line < a.start.line || b.end.line > a.end.line {
        return false;
    }
    if b.start.line == a.start.line && b.start.character < a.start.character {
        return false;
    }
    if b.end.line == a.end.line && b.end.character > a.end.character {
        return false;
    }

    true
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

/// Read a file from its path in a lossy way. If the file non UTF-8 characters, they will be replaced
/// by a �.
///
/// # Arguments
///
/// * `path` - [Path][PathBuf] of the file.
pub fn read_to_string_lossy(path: PathBuf) -> anyhow::Result<String> {
    let mut file = File::open(path)?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;

    Ok(String::from_utf8_lossy(&buf).to_string())
}
