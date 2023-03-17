use lsp_types::Position;

use crate::document::Document;

#[allow(unused)]
#[derive(Debug, Clone, Default)]
pub(crate) struct LexerState {
    pub(crate) line: u32,
    pub(crate) character: u32,
    pub(crate) parenthesis_count: i32,
    pub(crate) bracket_count: i32,
    pub(crate) braces_count: i32,
    pub(crate) literal: Option<Literal>,
    first_char: bool,
    prev: char,
    cur: char,
    is_escaped: bool,
}

impl LexerState {
    fn increment_line_count(&mut self) {
        self.line += 1;
        self.character = 0;
        self.first_char = true;
    }

    fn increment_character_count(&mut self) {
        if self.first_char {
            self.first_char = false;
        } else {
            self.character += 1;
        }
    }
}

impl PartialEq<Position> for LexerState {
    fn eq(&self, pos: &Position) -> bool {
        self.line == pos.line && self.character == pos.character
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub(crate) enum Literal {
    LineComment,
    BlockComment,
    DoubleQuotedString,
    SingleQuotedString,
    PreprocStatement,
}

impl Document {
    pub(crate) fn get_lexer_state(&self, pos: Position) -> Option<Literal> {
        let mut state = LexerState {
            first_char: true,
            ..Default::default()
        };
        for char_ in self.text.chars() {
            if char_ == '\n' {
                state.increment_line_count();
            }
            state.increment_character_count();
            state.prev = state.cur;
            state.cur = char_;

            match state.literal {
                Some(literal) => match literal {
                    Literal::PreprocStatement => match char_ {
                        '/' => {
                            if state.prev == '/' {
                                state.literal = Some(Literal::LineComment);
                            }
                        }
                        '\n' => {
                            if state.prev != '\\' {
                                state.literal = None;
                            }
                        }
                        _ => {}
                    },
                    Literal::LineComment => {
                        if char_ == '\n' {
                            state.literal = None;
                        }
                    }
                    Literal::BlockComment => {
                        if char_ == '/' && state.prev == '*' {
                            state.literal = None;
                        }
                    }
                    Literal::DoubleQuotedString => match char_ {
                        '\\' => state.is_escaped = !state.is_escaped,
                        '"' => {
                            if !state.is_escaped {
                                state.literal = None;
                            }
                        }
                        _ => {}
                    },
                    Literal::SingleQuotedString => match char_ {
                        '\\' => state.is_escaped = !state.is_escaped,
                        '\'' => {
                            if !state.is_escaped {
                                state.literal = None;
                            }
                        }
                        _ => {}
                    },
                },
                None => match char_ {
                    '/' => {
                        if state.prev == '/' {
                            state.literal = Some(Literal::LineComment);
                        }
                    }
                    '*' => {
                        if state.prev == '/' {
                            state.literal = Some(Literal::BlockComment);
                        }
                    }
                    '"' => state.literal = Some(Literal::DoubleQuotedString),
                    '\'' => state.literal = Some(Literal::SingleQuotedString),
                    '#' => state.literal = Some(Literal::PreprocStatement),
                    '(' => state.parenthesis_count += 1,
                    ')' => state.parenthesis_count -= 1,
                    '{' => state.braces_count += 1,
                    '}' => state.braces_count -= 1,
                    '[' => state.bracket_count += 1,
                    ']' => state.bracket_count -= 1,
                    _ => {}
                },
            }

            if state == pos {
                return state.literal;
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Literal, tests::fixtures::Fixture};

    #[test]
    fn double_quoted_string_inside() {
        let fixture = Fixture::parse(
            r#"
//! /main.sp
char foo[64] = "Hello World!";
                 |
^"#,
        );
        let pos = fixture.documents[0].cursor.unwrap();

        assert_eq!(
            fixture.documents[0]
                .to_document()
                .get_lexer_state(pos)
                .unwrap(),
            Literal::DoubleQuotedString
        );
    }

    #[test]
    fn double_quoted_string_after() {
        let fixture = Fixture::parse(
            r#"
//! /main.sp
char foo[64] = "Hello World!";
                             |
^"#,
        );
        let pos = fixture.documents[0].cursor.unwrap();

        assert_eq!(
            fixture.documents[0].to_document().get_lexer_state(pos),
            None
        );
    }

    #[test]
    fn single_quoted_string_inside() {
        let fixture = Fixture::parse(
            r#"
//! /main.sp
char foo[64] = 'Hello World!';
                 |
^"#,
        );
        let pos = fixture.documents[0].cursor.unwrap();

        assert_eq!(
            fixture.documents[0]
                .to_document()
                .get_lexer_state(pos)
                .unwrap(),
            Literal::SingleQuotedString
        );
    }

    #[test]
    fn single_quoted_string_after() {
        let fixture = Fixture::parse(
            r#"
//! /main.sp
char foo[64] = 'Hello World!';
                             |
^"#,
        );
        let pos = fixture.documents[0].cursor.unwrap();

        assert_eq!(
            fixture.documents[0].to_document().get_lexer_state(pos),
            None
        );
    }

    #[test]
    fn line_comment_inside() {
        let fixture = Fixture::parse(
            r#"
//! /main.sp
char foo[64] = "Hello World!"; // A comment
                                    |
"#,
        );
        let pos = fixture.documents[0].cursor.unwrap();

        assert_eq!(
            fixture.documents[0]
                .to_document()
                .get_lexer_state(pos)
                .unwrap(),
            Literal::LineComment
        );
    }

    #[test]
    fn line_comment_after() {
        let fixture = Fixture::parse(
            r#"
//! /main.sp
char foo[64] = "Hello World!"; // A comment

|
"#,
        );
        let pos = fixture.documents[0].cursor.unwrap();

        assert_eq!(
            fixture.documents[0].to_document().get_lexer_state(pos),
            None
        );
    }

    #[test]
    fn block_comment_inside() {
        let fixture = Fixture::parse(
            r#"
//! /main.sp
char foo[64] = "Hello World!"; /* A comment
                                   |
*/
"#,
        );
        let pos = fixture.documents[0].cursor.unwrap();

        assert_eq!(
            fixture.documents[0]
                .to_document()
                .get_lexer_state(pos)
                .unwrap(),
            Literal::BlockComment
        );
    }

    #[test]
    fn block_comment_after() {
        let fixture = Fixture::parse(
            r#"
//! /main.sp
char foo[64] = "Hello World!"; /* A comment
*/     
  |
"#,
        );
        let pos = fixture.documents[0].cursor.unwrap();

        assert_eq!(
            fixture.documents[0].to_document().get_lexer_state(pos),
            None
        );
    }

    #[test]
    fn preproc_statement_inside() {
        let fixture = Fixture::parse(
            r#"
//! /main.sp
#define FOO = "Hello World"
  |
"#,
        );
        let pos = fixture.documents[0].cursor.unwrap();

        assert_eq!(
            fixture.documents[0]
                .to_document()
                .get_lexer_state(pos)
                .unwrap(),
            Literal::PreprocStatement
        );
    }

    #[test]
    fn preproc_statement_after() {
        let fixture = Fixture::parse(
            r#"
//! /main.sp
#define FOO = "Hello World"

  |
"#,
        );
        let pos = fixture.documents[0].cursor.unwrap();

        assert_eq!(
            fixture.documents[0].to_document().get_lexer_state(pos),
            None
        );
    }

    #[test]
    fn preproc_statement_line_continuation() {
        let fixture = Fixture::parse(
            r#"
//! /main.sp
#define FOO =\
"Hello World"
  |
"#,
        );
        let pos = fixture.documents[0].cursor.unwrap();

        assert_eq!(
            fixture.documents[0]
                .to_document()
                .get_lexer_state(pos)
                .unwrap(),
            Literal::PreprocStatement
        );
    }
}
