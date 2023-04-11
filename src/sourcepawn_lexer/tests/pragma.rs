mod utils;

use crate::utils::assert_token_eq;
use sourcepawn_lexer::{Range, SourcePawnLexer, Symbol, TokenKind};

#[test]
fn pragma_simple() {
    let input = r#"#pragma deprecated foo
"#;

    let mut lexer = SourcePawnLexer::new(input);
    assert_token_eq!(lexer, MPragma, "#pragma deprecated foo", 0, 0, 0, 22);
    assert_token_eq!(lexer, Newline, "\n", 0, 22, 1, 0);
}

#[test]
fn pragma_no_line_break() {
    let input = "#pragma deprecated foo";

    let mut lexer = SourcePawnLexer::new(input);
    assert_token_eq!(lexer, MPragma, "#pragma deprecated foo", 0, 0, 0, 22);
}

#[test]
fn pragma_trailing_line_comment() {
    let input = r#"#pragma deprecated foo //bar
"#;

    let mut lexer = SourcePawnLexer::new(input);
    assert_token_eq!(lexer, MPragma, "#pragma deprecated foo ", 0, 0, 0, 23);
    assert_token_eq!(lexer, LineComment, "//bar", 0, 23, 0, 28);
}

#[test]
fn pragma_trailing_block_comment() {
    let input = r#"#pragma deprecated foo /* */
"#;

    let mut lexer = SourcePawnLexer::new(input);
    assert_token_eq!(lexer, MPragma, "#pragma deprecated foo ", 0, 0, 0, 23);
    assert_token_eq!(lexer, BlockComment, "/* */", 0, 23, 0, 28);
    assert_token_eq!(lexer, Newline, "\n", 0, 28, 1, 0);
}

#[test]
fn pragma_with_block_comment() {
    let input = r#"#pragma deprecated foo /* */ bar
"#;

    let mut lexer = SourcePawnLexer::new(input);
    assert_token_eq!(
        lexer,
        MPragma,
        "#pragma deprecated foo /* */ bar",
        0,
        0,
        0,
        32
    );
    assert_token_eq!(lexer, Newline, "\n", 0, 32, 1, 0);
}

#[test]
fn pragma_with_block_comment_and_line_continuation() {
    let input = r#"#pragma deprecated foo /* */ \
bar
"#;

    let mut lexer = SourcePawnLexer::new(input);
    assert_token_eq!(
        lexer,
        MPragma,
        "#pragma deprecated foo /* */ \\\nbar",
        0,
        0,
        1,
        4
    );
    assert_token_eq!(lexer, Newline, "\n", 1, 4, 2, 0);
}

#[test]
fn pragma_with_trailing_multiline_block_comment() {
    let input = r#"#pragma deprecated foo /*
*/ bar
"#;

    let mut lexer = SourcePawnLexer::new(input);
    assert_token_eq!(lexer, MPragma, "#pragma deprecated foo ", 0, 0, 0, 23);
    assert_token_eq!(lexer, BlockComment, "/*\n*/", 0, 23, 1, 3);
    assert_token_eq!(lexer, Identifier, "bar", 1, 4, 1, 7);
    assert_token_eq!(lexer, Newline, "\n", 1, 7, 2, 0);
}

#[test]
fn pragma_with_trailing_line_continuated_multiline_block_comment() {
    let input = r#"#pragma deprecated foo /* \
*/ bar
"#;

    let mut lexer = SourcePawnLexer::new(input);
    assert_token_eq!(
        lexer,
        MPragma,
        "#pragma deprecated foo /* \\\n*/ bar",
        0,
        0,
        1,
        7
    );
    assert_token_eq!(lexer, Newline, "\n", 1, 7, 2, 0);
}

#[test]
fn pragma_line_continuation() {
    let input = r#"#pragma deprecated foo \
bar
"#;

    let mut lexer = SourcePawnLexer::new(input);
    assert_token_eq!(lexer, MPragma, "#pragma deprecated foo \\\nbar", 0, 0, 1, 4);
}

#[test]
fn pragma_line_continuation_carriage_return() {
    let input = "#pragma deprecated foo \\\r\nbar\n";

    let mut lexer = SourcePawnLexer::new(input);
    assert_token_eq!(
        lexer,
        MPragma,
        "#pragma deprecated foo \\\r\nbar",
        0,
        0,
        1,
        4
    );
    assert_token_eq!(lexer, Newline, "\n", 1, 4, 2, 0);
}
