use logos::{Lexer, Logos};

use crate::lexer::Token;

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
        println!("{:?} {:?}", &token, self.lexer.span());
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

#[derive(Debug)]
pub enum TokenKind {
    Identifier,

    IntegerLiteral,

    HexLiteral,

    BinaryLiteral,

    OctodecimalLiteral,

    StringLiteral,

    CharLiteral,

    FloatLiteral,

    Newline,

    LineContinuation,

    LineComment,

    BlockComment,

    Bool,

    Break,

    Case,

    Char,

    Class,

    Const,

    Continue,

    Decl,

    Default,

    Defined,

    Delete,

    Do,

    Else,

    Enum,

    False,

    Float,

    For,

    Forward,

    Functag,

    Function,

    If,

    Int,

    InvalidFunction,

    Methodmap,

    Native,

    Null,

    New,

    Object,

    Property,

    Public,

    Return,

    Sizeof,

    Static,

    Stock,

    Struct,

    Switch,

    This,

    True,

    Typedef,

    Typeset,

    Union,

    Using,

    ViewAs,

    Void,

    While,

    Nullable,

    MDefine,

    MDeprecate,

    MElse,

    MEndif,

    MEndinput,

    MFile,

    MIf,

    MInclude,

    MLeaving,

    MLine,

    MOptionalNewdecls,

    MOptionalSemi,

    MPragma,

    MRequireNewdecls,

    MRequireSemi,

    MTryinclude,

    MUndef,

    Intrinsics,

    Ellipses,

    Plus,

    Minus,

    Star,

    Slash,

    Percent,

    Ampersand,

    Bitor,

    Bitxor,

    Shr,

    Ushr,

    Shl,

    Assign,

    Semicolon,

    LBrace,

    RBrace,

    LParen,

    RParen,

    LBracket,

    RBracket,

    AssignAdd,

    AssignSub,

    AssignMul,

    AssignDiv,

    AssignMod,

    AssignBitAnd,

    AssignBitOr,

    AssignBitXor,

    AssignShr,

    AssignUshl,

    AssignShl,

    Increment,

    Decrement,

    Equals,

    NotEquals,

    Lt,

    Le,

    Gt,

    Ge,

    And,

    Or,

    Comma,

    Not,

    Tilde,

    Qmark,

    Colon,

    Scope,

    Dot,

    Unknown,
}
