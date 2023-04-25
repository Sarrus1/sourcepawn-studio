mod utils;

use crate::utils::assert_token_eq;
use sourcepawn_lexer::*;

#[test]
fn pragma_simple() {
    let input = r#"#pragma deprecated foo
"#;

    let mut lexer = SourcepawnLexer::new(input);
    assert_token_eq!(
        lexer,
        TokenKind::PreprocDir(PreprocDir::MPragma),
        "#pragma deprecated foo",
        0,
        0,
        0,
        22,
        0,
        0
    );
    assert_token_eq!(lexer, TokenKind::Newline, "\n", 0, 22, 1, 0, 0, 0);
    assert_token_eq!(lexer, TokenKind::Eof, "\0", 1, 0, 1, 0, 0, 0);
}

#[test]
fn pragma_no_line_break() {
    let input = "#pragma deprecated foo";

    let mut lexer = SourcepawnLexer::new(input);
    assert_token_eq!(
        lexer,
        TokenKind::PreprocDir(PreprocDir::MPragma),
        "#pragma deprecated foo",
        0,
        0,
        0,
        22,
        0,
        0
    );
    assert_token_eq!(lexer, TokenKind::Eof, "\0", 0, 22, 0, 22, 0, 0);
}

#[test]
fn pragma_trailing_line_comment() {
    let input = r#"#pragma deprecated foo //bar
"#;

    let mut lexer = SourcepawnLexer::new(input);
    assert_token_eq!(
        lexer,
        TokenKind::PreprocDir(PreprocDir::MPragma),
        "#pragma deprecated foo ",
        0,
        0,
        0,
        23,
        0,
        0
    );
    assert_token_eq!(
        lexer,
        TokenKind::Comment(Comment::LineComment),
        "//bar",
        0,
        23,
        0,
        28,
        0,
        0
    );
    assert_token_eq!(lexer, TokenKind::Newline, "\n", 0, 28, 1, 0, 0, 0);
    assert_token_eq!(lexer, TokenKind::Eof, "\0", 1, 0, 1, 0, 0, 0);
}

#[test]
fn pragma_trailing_block_comment() {
    let input = r#"#pragma deprecated foo /* */
"#;

    let mut lexer = SourcepawnLexer::new(input);
    assert_token_eq!(
        lexer,
        TokenKind::PreprocDir(PreprocDir::MPragma),
        "#pragma deprecated foo ",
        0,
        0,
        0,
        23,
        0,
        0
    );
    assert_token_eq!(
        lexer,
        TokenKind::Comment(Comment::BlockComment),
        "/* */",
        0,
        23,
        0,
        28,
        0,
        0
    );
    assert_token_eq!(lexer, TokenKind::Newline, "\n", 0, 28, 1, 0, 0, 0);
    assert_token_eq!(lexer, TokenKind::Eof, "\0", 1, 0, 1, 0, 0, 0);
}

#[test]
fn pragma_with_block_comment() {
    let input = r#"#pragma deprecated foo /* */ bar
"#;

    let mut lexer = SourcepawnLexer::new(input);
    assert_token_eq!(
        lexer,
        TokenKind::PreprocDir(PreprocDir::MPragma),
        "#pragma deprecated foo /* */ bar",
        0,
        0,
        0,
        32,
        0,
        0
    );
    assert_token_eq!(lexer, TokenKind::Newline, "\n", 0, 32, 1, 0, 0, 0);
    assert_token_eq!(lexer, TokenKind::Eof, "\0", 1, 0, 1, 0, 0, 0);
}

#[test]
fn pragma_with_block_comment_and_line_continuation() {
    let input = r#"#pragma deprecated foo /* */ \
bar
"#;

    let mut lexer = SourcepawnLexer::new(input);
    assert_token_eq!(
        lexer,
        TokenKind::PreprocDir(PreprocDir::MPragma),
        "#pragma deprecated foo /* */ \\\nbar",
        0,
        0,
        1,
        3,
        0,
        0
    );
    assert_token_eq!(lexer, TokenKind::Newline, "\n", 1, 3, 2, 0, 0, 0);
    assert_token_eq!(lexer, TokenKind::Eof, "\0", 2, 0, 2, 0, 0, 0);
}

#[test]
fn pragma_with_trailing_multiline_block_comment() {
    let input = r#"#pragma deprecated foo /*
*/ bar
"#;

    let mut lexer = SourcepawnLexer::new(input);
    assert_token_eq!(
        lexer,
        TokenKind::PreprocDir(PreprocDir::MPragma),
        "#pragma deprecated foo ",
        0,
        0,
        0,
        23,
        0,
        0
    );
    assert_token_eq!(
        lexer,
        TokenKind::Comment(Comment::BlockComment),
        "/*\n*/",
        0,
        23,
        1,
        3,
        0,
        0
    );
    assert_token_eq!(lexer, TokenKind::Identifier, "bar", 1, 4, 1, 7, 0, 1);
    assert_token_eq!(lexer, TokenKind::Newline, "\n", 1, 7, 2, 0, 0, 0);
    assert_token_eq!(lexer, TokenKind::Eof, "\0", 2, 0, 2, 0, 0, 0);
}

#[test]
fn pragma_with_trailing_line_continuated_multiline_block_comment() {
    let input = r#"#pragma deprecated foo /* \
*/ bar
"#;

    let mut lexer = SourcepawnLexer::new(input);
    assert_token_eq!(
        lexer,
        TokenKind::PreprocDir(PreprocDir::MPragma),
        "#pragma deprecated foo /* \\\n*/ bar",
        0,
        0,
        1,
        6,
        0,
        0
    );
    assert_token_eq!(lexer, TokenKind::Newline, "\n", 1, 6, 2, 0, 0, 0);
    assert_token_eq!(lexer, TokenKind::Eof, "\0", 2, 0, 2, 0, 0, 0);
}

#[test]
fn pragma_line_continuation() {
    let input = r#"#pragma deprecated foo \
bar
"#;

    let mut lexer = SourcepawnLexer::new(input);
    assert_token_eq!(
        lexer,
        TokenKind::PreprocDir(PreprocDir::MPragma),
        "#pragma deprecated foo \\\nbar",
        0,
        0,
        1,
        3,
        0,
        0
    );
    assert_token_eq!(lexer, TokenKind::Newline, "\n", 1, 3, 2, 0, 0, 0);
    assert_token_eq!(lexer, TokenKind::Eof, "\0", 2, 0, 2, 0, 0, 0);
}

#[test]
fn pragma_line_continuation_carriage_return() {
    let input = "#pragma deprecated foo \\\r\nbar\n";

    let mut lexer = SourcepawnLexer::new(input);
    assert_token_eq!(
        lexer,
        TokenKind::PreprocDir(PreprocDir::MPragma),
        "#pragma deprecated foo \\\r\nbar",
        0,
        0,
        1,
        3,
        0,
        0
    );
    assert_token_eq!(lexer, TokenKind::Newline, "\n", 1, 3, 2, 0, 0, 0);
    assert_token_eq!(lexer, TokenKind::Eof, "\0", 2, 0, 2, 0, 0, 0);
}
