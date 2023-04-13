use lazy_static::lazy_static;
use logos::{Lexer, Logos};
use regex::Regex;

use crate::{token::Token, token_kind::TokenKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub start_line: usize,
    pub end_line: usize,
    pub start_col: usize,
    pub end_col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub token_kind: TokenKind,
    pub text: String,
    pub range: Range,
}

#[derive(Debug, Clone)]
pub struct SourcePawnLexer<'a> {
    lexer: Lexer<'a, Token>,
    line_number: usize,
    line_span_start: usize,
    in_preprocessor: bool,
}

impl SourcePawnLexer<'_> {
    pub fn new(input: &str) -> SourcePawnLexer {
        SourcePawnLexer {
            lexer: Token::lexer(input),
            line_number: 0,
            line_span_start: 0,
            in_preprocessor: false,
        }
    }

    pub fn in_preprocessor(&self) -> bool {
        self.in_preprocessor
    }
}

impl Iterator for SourcePawnLexer<'_> {
    type Item = Symbol;

    fn next(&mut self) -> Option<Symbol> {
        lazy_static! {
            static ref RE1: Regex = Regex::new(r"\n").unwrap();
        }
        lazy_static! {
            static ref RE2: Regex = Regex::new(r"\\\r?\n").unwrap();
        }
        let token = self.lexer.next()?;

        let start_line = self.line_number;
        let start_col = self.lexer.span().start - self.line_span_start;
        let text = self.lexer.slice().to_string();
        match token {
            Token::StringLiteral | Token::BlockComment | Token::MPragma => {
                if token == Token::MPragma {
                    self.in_preprocessor = true;
                }
                let line_breaks: Vec<_> = RE1.find_iter(text.as_str()).collect();
                let line_continuations: Vec<_> = RE2.find_iter(text.as_str()).collect();

                if let Some(last) = line_continuations.last() {
                    self.line_number += line_breaks.len();
                    self.line_span_start = self.lexer.span().start + last.end();
                } else if let Some(last) = line_breaks.last() {
                    self.in_preprocessor = false;
                    self.line_number += line_breaks.len();
                    self.line_span_start = self.lexer.span().start + last.start();
                }
            }
            Token::MDefine
            | Token::MDeprecate
            | Token::MIf
            | Token::MElse
            | Token::MEndinput
            | Token::MFile
            | Token::MOptionalNewdecls
            | Token::MOptionalSemi
            | Token::MRequireNewdecls
            | Token::MRequireSemi
            | Token::MTryinclude
            | Token::MUndef
            | Token::MEndif
            | Token::MInclude
            | Token::MLeaving => self.in_preprocessor = true,
            Token::LineContinuation => {
                self.line_number += 1;
                self.line_span_start = self.lexer.span().end;
            }
            Token::Newline => {
                self.in_preprocessor = false;
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
