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

/// The intersection of two ranges.
///
/// # Arguments
/// * `range_a` - The first range.
/// * `range_b` - The second range.
pub fn intersect(range_a: lsp_types::Range, range_b: lsp_types::Range) -> Option<lsp_types::Range> {
    let start = if range_a.start.line > range_b.start.line
        || (range_a.start.line == range_b.start.line
            && range_a.start.character > range_b.start.character)
    {
        range_a.start
    } else {
        range_b.start
    };

    let end = if range_a.end.line < range_b.end.line
        || (range_a.end.line == range_b.end.line && range_a.end.character < range_b.end.character)
    {
        range_a.end
    } else {
        range_b.end
    };

    if start.line > end.line || (start.line == end.line && start.character > end.character) {
        None
    } else {
        Some(lsp_types::Range { start, end })
    }
}

#[test]
fn intersect_1() {
    let range_a = lsp_types::Range {
        start: lsp_types::Position {
            line: 1,
            character: 1,
        },
        end: lsp_types::Position {
            line: 2,
            character: 2,
        },
    };
    let range_b = lsp_types::Range {
        start: lsp_types::Position {
            line: 0,
            character: 0,
        },
        end: lsp_types::Position {
            line: 3,
            character: 3,
        },
    };
    let result = intersect(range_a, range_b);
    assert_eq!(
        result,
        Some(lsp_types::Range {
            start: lsp_types::Position {
                line: 1,
                character: 1
            },
            end: lsp_types::Position {
                line: 2,
                character: 2
            }
        })
    );
}

#[test]
fn intersect_2() {
    let range_a = lsp_types::Range {
        start: lsp_types::Position {
            line: 1,
            character: 1,
        },
        end: lsp_types::Position {
            line: 4,
            character: 4,
        },
    };
    let range_b = lsp_types::Range {
        start: lsp_types::Position {
            line: 0,
            character: 0,
        },
        end: lsp_types::Position {
            line: 3,
            character: 3,
        },
    };
    let result = intersect(range_a, range_b);
    assert_eq!(
        result,
        Some(lsp_types::Range {
            start: lsp_types::Position {
                line: 1,
                character: 1
            },
            end: lsp_types::Position {
                line: 3,
                character: 3
            }
        })
    );
}

#[test]
fn intersect_3() {
    let range_a = lsp_types::Range {
        start: lsp_types::Position {
            line: 1,
            character: 1,
        },
        end: lsp_types::Position {
            line: 2,
            character: 2,
        },
    };
    let range_b = lsp_types::Range {
        start: lsp_types::Position {
            line: 2,
            character: 2,
        },
        end: lsp_types::Position {
            line: 3,
            character: 3,
        },
    };
    let result = intersect(range_a, range_b);
    assert_eq!(
        result,
        Some(lsp_types::Range {
            start: lsp_types::Position {
                line: 2,
                character: 2
            },
            end: lsp_types::Position {
                line: 2,
                character: 2
            }
        })
    );
}
