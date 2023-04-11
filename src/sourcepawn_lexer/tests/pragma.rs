use logos::Logos;
use sourcepawn_lexer::token::Token;

#[test]
fn pragma_simple() {
    let input = r#"#pragma deprecated foo
"#;

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MPragma));
    assert_eq!(lexer.span(), 0..22);
    assert_eq!(lexer.slice(), "#pragma deprecated foo");

    assert_eq!(lexer.next(), Some(Token::Newline));
    assert_eq!(lexer.span(), 22..23);
    assert_eq!(lexer.slice(), "\n");
}

#[test]
fn pragma_no_line_break() {
    let input = "#pragma deprecated foo";

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MPragma));
    assert_eq!(lexer.span(), 0..22);
    assert_eq!(lexer.slice(), "#pragma deprecated foo");
}

#[test]
fn pragma_trailing_line_comment() {
    let input = r#"#pragma deprecated foo //bar
"#;

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MPragma));
    assert_eq!(lexer.span(), 0..23);
    assert_eq!(lexer.slice(), "#pragma deprecated foo ");

    assert_eq!(lexer.next(), Some(Token::LineComment));
    assert_eq!(lexer.span(), 23..28);
    assert_eq!(lexer.slice(), "//bar");

    assert_eq!(lexer.next(), Some(Token::Newline));
    assert_eq!(lexer.span(), 28..29);
    assert_eq!(lexer.slice(), "\n");
}

#[test]
fn pragma_trailing_block_comment() {
    let input = r#"#pragma deprecated foo /* */
"#;

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MPragma));
    assert_eq!(lexer.span(), 0..23);
    assert_eq!(lexer.slice(), "#pragma deprecated foo ");

    assert_eq!(lexer.next(), Some(Token::BlockComment));
    assert_eq!(lexer.span(), 23..28);
    assert_eq!(lexer.slice(), "/* */");

    assert_eq!(lexer.next(), Some(Token::Newline));
    assert_eq!(lexer.span(), 28..29);
    assert_eq!(lexer.slice(), "\n");
}

#[test]
fn pragma_with_block_comment() {
    let input = r#"#pragma deprecated foo /* */ bar
"#;

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MPragma));
    assert_eq!(lexer.span(), 0..32);
    assert_eq!(lexer.slice(), "#pragma deprecated foo /* */ bar");

    assert_eq!(lexer.next(), Some(Token::Newline));
    assert_eq!(lexer.span(), 32..33);
    assert_eq!(lexer.slice(), "\n");
}

#[test]
fn pragma_with_block_comment_and_line_continuation() {
    let input = r#"#pragma deprecated foo /* */ \
bar
"#;

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MPragma));
    assert_eq!(lexer.span(), 0..34);
    assert_eq!(lexer.slice(), "#pragma deprecated foo /* */ \\\nbar");

    assert_eq!(lexer.next(), Some(Token::Newline));
    assert_eq!(lexer.span(), 34..35);
    assert_eq!(lexer.slice(), "\n");
}

#[test]
fn pragma_with_trailing_multiline_block_comment() {
    let input = r#"#pragma deprecated foo /* 
*/ bar
"#;

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MPragma));
    assert_eq!(lexer.span(), 0..23);
    assert_eq!(lexer.slice(), "#pragma deprecated foo ");

    assert_eq!(lexer.next(), Some(Token::BlockComment));
    assert_eq!(lexer.span(), 23..29);
    assert_eq!(lexer.slice(), "/* \n*/");

    assert_eq!(lexer.next(), Some(Token::Identifier));
    assert_eq!(lexer.span(), 30..33);
    assert_eq!(lexer.slice(), "bar");

    assert_eq!(lexer.next(), Some(Token::Newline));
    assert_eq!(lexer.span(), 33..34);
    assert_eq!(lexer.slice(), "\n");
}

#[test]
fn pragma_with_trailing_line_continuated_multiline_block_comment() {
    let input = r#"#pragma deprecated foo /* \
*/ bar
"#;

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MPragma));
    assert_eq!(lexer.span(), 0..34);
    assert_eq!(lexer.slice(), "#pragma deprecated foo /* \\\n*/ bar");

    assert_eq!(lexer.next(), Some(Token::Newline));
    assert_eq!(lexer.span(), 34..35);
    assert_eq!(lexer.slice(), "\n");
}

#[test]
fn pragma_line_continuation() {
    let input = r#"#pragma deprecated foo \
bar
"#;

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MPragma));
    assert_eq!(lexer.span(), 0..28);
    assert_eq!(lexer.slice(), "#pragma deprecated foo \\\nbar");

    assert_eq!(lexer.next(), Some(Token::Newline));
    assert_eq!(lexer.span(), 28..29);
    assert_eq!(lexer.slice(), "\n");
}

#[test]
fn pragma_line_continuation_carriage_return() {
    let input = "#pragma deprecated foo \\\r\nbar\n";

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MPragma));
    assert_eq!(lexer.span(), 0..29);
    assert_eq!(lexer.slice(), "#pragma deprecated foo \\\r\nbar");

    assert_eq!(lexer.next(), Some(Token::Newline));
    assert_eq!(lexer.span(), 29..30);
    assert_eq!(lexer.slice(), "\n");
}
