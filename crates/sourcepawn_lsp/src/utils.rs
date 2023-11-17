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

pub fn prettify_s_expression(s_expr: &str) -> String {
    let mut result = String::new();
    let mut indent_level = 0;

    for c in s_expr.chars() {
        match c {
            '(' => {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&" ".repeat(indent_level));
                result.push(c);
                indent_level += 2;
            }
            ')' => {
                indent_level -= 2;
                result.push('\n');
                result.push_str(&" ".repeat(indent_level));
                result.push(c);
            }
            ' ' => {
                result.push(' ');
            }
            _ => {
                result.push(c);
            }
        }
    }

    result
}
