use logos::Logos;
use sourcepawn_lexer::token::Token;

#[test]
fn define_simple() {
    let input = r#"#define FOO 1
"#;

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MDefine));
    assert_eq!(lexer.span(), 0..7);
    assert_eq!(lexer.slice(), "#define");

    assert_eq!(lexer.next(), Some(Token::Identifier));
    assert_eq!(lexer.span(), 8..11);
    assert_eq!(lexer.slice(), "FOO");

    assert_eq!(lexer.next(), Some(Token::IntegerLiteral));
    assert_eq!(lexer.span(), 12..13);
    assert_eq!(lexer.slice(), "1");

    assert_eq!(lexer.next(), Some(Token::Newline));
    assert_eq!(lexer.span(), 13..14);
    assert_eq!(lexer.slice(), "\n");
}

#[test]
fn define_no_value() {
    let input = r#"#define FOO
"#;

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MDefine));
    assert_eq!(lexer.span(), 0..7);
    assert_eq!(lexer.slice(), "#define");

    assert_eq!(lexer.next(), Some(Token::Identifier));
    assert_eq!(lexer.span(), 8..11);
    assert_eq!(lexer.slice(), "FOO");

    assert_eq!(lexer.next(), Some(Token::Newline));
    assert_eq!(lexer.span(), 11..12);
    assert_eq!(lexer.slice(), "\n");
}

#[test]
fn define_no_line_break() {
    let input = "#define FOO 1";

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MDefine));
    assert_eq!(lexer.span(), 0..7);
    assert_eq!(lexer.slice(), "#define");

    assert_eq!(lexer.next(), Some(Token::Identifier));
    assert_eq!(lexer.span(), 8..11);
    assert_eq!(lexer.slice(), "FOO");

    assert_eq!(lexer.next(), Some(Token::IntegerLiteral));
    assert_eq!(lexer.span(), 12..13);
    assert_eq!(lexer.slice(), "1");
}

#[test]
fn define_trailing_line_comment() {
    let input = r#"#define FOO 1 //bar
"#;

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MDefine));
    assert_eq!(lexer.span(), 0..7);
    assert_eq!(lexer.slice(), "#define");

    assert_eq!(lexer.next(), Some(Token::Identifier));
    assert_eq!(lexer.span(), 8..11);
    assert_eq!(lexer.slice(), "FOO");

    assert_eq!(lexer.next(), Some(Token::IntegerLiteral));
    assert_eq!(lexer.span(), 12..13);
    assert_eq!(lexer.slice(), "1");

    assert_eq!(lexer.next(), Some(Token::LineComment));
    assert_eq!(lexer.span(), 14..19);
    assert_eq!(lexer.slice(), "//bar");

    assert_eq!(lexer.next(), Some(Token::Newline));
    assert_eq!(lexer.span(), 19..20);
    assert_eq!(lexer.slice(), "\n");
}

#[test]
fn define_trailing_block_comment() {
    let input = r#"#define FOO 1 /* */
"#;

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MDefine));
    assert_eq!(lexer.span(), 0..7);
    assert_eq!(lexer.slice(), "#define");

    assert_eq!(lexer.next(), Some(Token::Identifier));
    assert_eq!(lexer.span(), 8..11);
    assert_eq!(lexer.slice(), "FOO");

    assert_eq!(lexer.next(), Some(Token::IntegerLiteral));
    assert_eq!(lexer.span(), 12..13);
    assert_eq!(lexer.slice(), "1");

    assert_eq!(lexer.next(), Some(Token::BlockComment));
    assert_eq!(lexer.span(), 14..19);
    assert_eq!(lexer.slice(), "/* */");

    assert_eq!(lexer.next(), Some(Token::Newline));
    assert_eq!(lexer.span(), 19..20);
    assert_eq!(lexer.slice(), "\n");
}

#[test]
fn define_with_block_comment() {
    let input = r#"#define FOO 1 /* */ + 1
"#;

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MDefine));
    assert_eq!(lexer.span(), 0..7);
    assert_eq!(lexer.slice(), "#define");

    assert_eq!(lexer.next(), Some(Token::Identifier));
    assert_eq!(lexer.span(), 8..11);
    assert_eq!(lexer.slice(), "FOO");

    assert_eq!(lexer.next(), Some(Token::IntegerLiteral));
    assert_eq!(lexer.span(), 12..13);
    assert_eq!(lexer.slice(), "1");

    assert_eq!(lexer.next(), Some(Token::BlockComment));
    assert_eq!(lexer.span(), 14..19);
    assert_eq!(lexer.slice(), "/* */");

    assert_eq!(lexer.next(), Some(Token::Plus));
    assert_eq!(lexer.span(), 20..21);
    assert_eq!(lexer.slice(), "+");

    assert_eq!(lexer.next(), Some(Token::IntegerLiteral));
    assert_eq!(lexer.span(), 22..23);
    assert_eq!(lexer.slice(), "1");

    assert_eq!(lexer.next(), Some(Token::Newline));
    assert_eq!(lexer.span(), 23..24);
    assert_eq!(lexer.slice(), "\n");
}

#[test]
fn define_with_block_comment_and_line_continuation() {
    let input = r#"#define FOO 1 /* */ \
+ 1
"#;

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MDefine));
    assert_eq!(lexer.span(), 0..7);
    assert_eq!(lexer.slice(), "#define");

    assert_eq!(lexer.next(), Some(Token::Identifier));
    assert_eq!(lexer.span(), 8..11);
    assert_eq!(lexer.slice(), "FOO");

    assert_eq!(lexer.next(), Some(Token::IntegerLiteral));
    assert_eq!(lexer.span(), 12..13);
    assert_eq!(lexer.slice(), "1");

    assert_eq!(lexer.next(), Some(Token::BlockComment));
    assert_eq!(lexer.span(), 14..19);
    assert_eq!(lexer.slice(), "/* */");

    assert_eq!(lexer.next(), Some(Token::LineContinuation));
    assert_eq!(lexer.span(), 20..22);
    assert_eq!(lexer.slice(), "\\\n");

    assert_eq!(lexer.next(), Some(Token::Plus));
    assert_eq!(lexer.span(), 22..23);
    assert_eq!(lexer.slice(), "+");

    assert_eq!(lexer.next(), Some(Token::IntegerLiteral));
    assert_eq!(lexer.span(), 24..25);
    assert_eq!(lexer.slice(), "1");

    assert_eq!(lexer.next(), Some(Token::Newline));
    assert_eq!(lexer.span(), 25..26);
    assert_eq!(lexer.slice(), "\n");
}

#[test]
fn define_with_trailing_multiline_block_comment() {
    let input = r#"#define FOO 1 /* 
*/ + 1
"#;

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MDefine));
    assert_eq!(lexer.span(), 0..7);
    assert_eq!(lexer.slice(), "#define");

    assert_eq!(lexer.next(), Some(Token::Identifier));
    assert_eq!(lexer.span(), 8..11);
    assert_eq!(lexer.slice(), "FOO");

    assert_eq!(lexer.next(), Some(Token::IntegerLiteral));
    assert_eq!(lexer.span(), 12..13);
    assert_eq!(lexer.slice(), "1");

    assert_eq!(lexer.next(), Some(Token::BlockComment));
    assert_eq!(lexer.span(), 14..20);
    assert_eq!(lexer.slice(), "/* \n*/");

    assert_eq!(lexer.next(), Some(Token::Plus));
    assert_eq!(lexer.span(), 21..22);
    assert_eq!(lexer.slice(), "+");

    assert_eq!(lexer.next(), Some(Token::IntegerLiteral));
    assert_eq!(lexer.span(), 23..24);
    assert_eq!(lexer.slice(), "1");

    assert_eq!(lexer.next(), Some(Token::Newline));
    assert_eq!(lexer.span(), 24..25);
    assert_eq!(lexer.slice(), "\n");
}

#[test]
fn define_with_trailing_line_continuated_multiline_block_comment() {
    let input = r#"#define FOO 1 /* \
*/ + 1
"#;

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MDefine));
    assert_eq!(lexer.span(), 0..7);
    assert_eq!(lexer.slice(), "#define");

    assert_eq!(lexer.next(), Some(Token::Identifier));
    assert_eq!(lexer.span(), 8..11);
    assert_eq!(lexer.slice(), "FOO");

    assert_eq!(lexer.next(), Some(Token::IntegerLiteral));
    assert_eq!(lexer.span(), 12..13);
    assert_eq!(lexer.slice(), "1");

    assert_eq!(lexer.next(), Some(Token::BlockComment));
    assert_eq!(lexer.span(), 14..21);
    assert_eq!(lexer.slice(), "/* \\\n*/");

    assert_eq!(lexer.next(), Some(Token::Plus));
    assert_eq!(lexer.span(), 22..23);
    assert_eq!(lexer.slice(), "+");

    assert_eq!(lexer.next(), Some(Token::IntegerLiteral));
    assert_eq!(lexer.span(), 24..25);
    assert_eq!(lexer.slice(), "1");

    assert_eq!(lexer.next(), Some(Token::Newline));
    assert_eq!(lexer.span(), 25..26);
    assert_eq!(lexer.slice(), "\n");
}

#[test]
fn define_line_continuation() {
    let input = r#"#define FOO 1 \
+ 1
"#;

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MDefine));
    assert_eq!(lexer.span(), 0..7);
    assert_eq!(lexer.slice(), "#define");

    assert_eq!(lexer.next(), Some(Token::Identifier));
    assert_eq!(lexer.span(), 8..11);
    assert_eq!(lexer.slice(), "FOO");

    assert_eq!(lexer.next(), Some(Token::IntegerLiteral));
    assert_eq!(lexer.span(), 12..13);
    assert_eq!(lexer.slice(), "1");

    assert_eq!(lexer.next(), Some(Token::LineContinuation));
    assert_eq!(lexer.span(), 14..16);
    assert_eq!(lexer.slice(), "\\\n");

    assert_eq!(lexer.next(), Some(Token::Plus));
    assert_eq!(lexer.span(), 16..17);
    assert_eq!(lexer.slice(), "+");

    assert_eq!(lexer.next(), Some(Token::IntegerLiteral));
    assert_eq!(lexer.span(), 18..19);
    assert_eq!(lexer.slice(), "1");

    assert_eq!(lexer.next(), Some(Token::Newline));
    assert_eq!(lexer.span(), 19..20);
    assert_eq!(lexer.slice(), "\n");
}

#[test]
fn define_line_continuation_carriage_return() {
    let input = "#define FOO 1 \\\r\n+ 1\n";

    let mut lexer = Token::lexer(input);
    assert_eq!(lexer.next(), Some(Token::MDefine));
    assert_eq!(lexer.span(), 0..7);
    assert_eq!(lexer.slice(), "#define");

    assert_eq!(lexer.next(), Some(Token::Identifier));
    assert_eq!(lexer.span(), 8..11);
    assert_eq!(lexer.slice(), "FOO");

    assert_eq!(lexer.next(), Some(Token::IntegerLiteral));
    assert_eq!(lexer.span(), 12..13);
    assert_eq!(lexer.slice(), "1");

    assert_eq!(lexer.next(), Some(Token::LineContinuation));
    assert_eq!(lexer.span(), 14..17);
    assert_eq!(lexer.slice(), "\\\r\n");

    assert_eq!(lexer.next(), Some(Token::Plus));
    assert_eq!(lexer.span(), 17..18);
    assert_eq!(lexer.slice(), "+");

    assert_eq!(lexer.next(), Some(Token::IntegerLiteral));
    assert_eq!(lexer.span(), 19..20);
    assert_eq!(lexer.slice(), "1");

    assert_eq!(lexer.next(), Some(Token::Newline));
    assert_eq!(lexer.span(), 20..21);
    assert_eq!(lexer.slice(), "\n");
}
