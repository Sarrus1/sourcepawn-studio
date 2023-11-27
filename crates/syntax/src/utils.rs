use lsp_types::Position;

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

/// Convert an LSP [Position](lsp_types::Position) to a Tree-sitter [Point](tree_sitter::Point).
///
/// # Arguments
///
/// * `pos` - LSP [Position](lsp_types::Position) to convert.
pub fn lsp_position_to_ts_point(pos: &lsp_types::Position) -> tree_sitter::Point {
    tree_sitter::Point {
        row: pos.line as usize,
        column: pos.character as usize,
    }
}

/// Convert a Tree-sitter [Point](tree_sitter::Point) to an LSP [Position](lsp_types::Position).
///
/// # Arguments
///
/// * `point` - Tree-sitter [Point](tree_sitter::Point) to convert.
pub fn point_to_lsp_position(point: &tree_sitter::Point) -> lsp_types::Position {
    Position::new(point.row as u32, point.column as u32)
}
