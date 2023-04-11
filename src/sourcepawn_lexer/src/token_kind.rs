use logos::{Lexer, Logos};

use crate::token::Token;

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
