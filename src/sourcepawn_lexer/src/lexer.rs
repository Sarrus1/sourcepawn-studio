use logos::{Lexer, Logos};

use crate::{token::Token, token_kind::TokenKind};

#[derive(Debug)]
pub struct Range {
    start_line: usize,
    end_line: usize,
    start_col: usize,
    end_col: usize,
}

#[derive(Debug)]
pub struct Symbol {
    pub token_kind: TokenKind,
    pub text: String,
    pub range: Range,
}

#[derive(Debug)]
pub struct SourcePawnLexer<'a> {
    lexer: Lexer<'a, Token>,
    line_number: usize,
    line_span_start: usize,
}

impl SourcePawnLexer<'_> {
    pub fn new(input: &str) -> SourcePawnLexer {
        SourcePawnLexer {
            lexer: Token::lexer(input),
            line_number: 1,
            line_span_start: 0,
        }
    }
}

impl Iterator for SourcePawnLexer<'_> {
    type Item = Symbol;

    fn next(&mut self) -> Option<Symbol> {
        let token = self.lexer.next()?;

        let start_line = self.line_number;
        let start_col = self.lexer.span().start - self.line_span_start;
        let text = self.lexer.slice().to_string();
        match token {
            Token::StringLiteral | Token::BlockComment | Token::MPragma => {
                let line_breaks: Vec<_> = text.match_indices('\n').collect();
                if let Some(last) = line_breaks.last() {
                    self.line_number += line_breaks.len();
                    self.line_span_start = self.lexer.span().start + last.0;
                }
            }
            Token::LineContinuation | Token::Newline => {
                self.line_number += 1;
                self.line_span_start = self.lexer.span().end;
            }
            _ => {}
        }
        let token_kind = TokenKind::try_from(token).unwrap();
        let range = Range {
            start_line,
            start_col,
            end_line: self.line_number,
            end_col: self.lexer.span().end - self.line_span_start,
        };
        Some(Symbol {
            token_kind,
            text,
            range,
        })
    }
}

impl TryFrom<Token> for TokenKind {
    type Error = &'static str;

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        let token_kind = match token {
            Token::Identifier => TokenKind::Identifier,
            Token::BlockComment => TokenKind::BlockComment,
            _ => TokenKind::Identifier,
        };

        Ok(token_kind)
    }
}
