mod utils;

use crate::utils::assert_token_eq;
use lsp_types::{Position, Range};
use sourcepawn_lexer::*;

#[test]
fn include_simple_1() {
    let input = r#"#include <sourcemod>
int foo;"#;

    let mut lexer = SourcepawnLexer::new(input);
    assert!(!lexer.in_preprocessor());
    assert_token_eq!(
        lexer,
        TokenKind::PreprocDir(PreprocDir::MInclude),
        "#include <sourcemod>",
        0,
        0,
        0,
        20,
        0,
        0
    );
    assert!(lexer.in_preprocessor());
    assert_token_eq!(lexer, TokenKind::Newline, "\n", 0, 20, 1, 0, 0, 0);
    assert!(!lexer.in_preprocessor());
    assert_token_eq!(lexer, TokenKind::Int, "int", 1, 0, 1, 3, 0, 0);
    assert!(!lexer.in_preprocessor());
    assert_token_eq!(lexer, TokenKind::Identifier, "foo", 1, 4, 1, 7, 0, 1);
    assert!(!lexer.in_preprocessor());
    assert_token_eq!(lexer, TokenKind::Semicolon, ";", 1, 7, 1, 8, 0, 0);
    assert!(!lexer.in_preprocessor());
    assert_token_eq!(lexer, TokenKind::Eof, "\0", 1, 8, 1, 8, 0, 0);
}

#[test]
fn opening_chevron_1() {
    let input = r#"1 < 2"#;

    let mut lexer = SourcepawnLexer::new(input);
    assert_token_eq!(
        lexer,
        TokenKind::Literal(Literal::IntegerLiteral),
        "1",
        0,
        0,
        0,
        1,
        0,
        0
    );
    assert_token_eq!(
        lexer,
        TokenKind::Operator(Operator::Lt),
        "<",
        0,
        2,
        0,
        3,
        0,
        1
    );
    assert_token_eq!(
        lexer,
        TokenKind::Literal(Literal::IntegerLiteral),
        "2",
        0,
        4,
        0,
        5,
        0,
        1
    );
    assert!(!lexer.in_preprocessor());
    assert_token_eq!(lexer, TokenKind::Eof, "\0", 0, 5, 0, 5, 0, 0);
}

#[test]
fn include_line_continuation_1() {
    let input = r#"#include <sourcemod\
>"#;

    let mut lexer = SourcepawnLexer::new(input);
    assert!(!lexer.in_preprocessor());
    assert_token_eq!(
        lexer,
        TokenKind::PreprocDir(PreprocDir::MInclude),
        "#include <sourcemod\\\n>",
        0,
        0,
        1,
        1,
        0,
        0
    );
    assert!(lexer.in_preprocessor());
    assert_token_eq!(lexer, TokenKind::Eof, "\0", 1, 1, 1, 1, 0, 0);
}
