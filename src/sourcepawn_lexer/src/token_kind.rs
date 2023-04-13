use crate::token::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    /// Integer literal.
    ///
    /// # Examples
    /// ```
    /// 1234
    /// 10_000_000
    /// ```
    IntegerLiteral,

    /// Hexadecimal literal.
    ///
    /// # Examples
    /// ```
    /// 0x1234
    /// ```
    HexLiteral,

    /// Binary literal.
    ///
    /// # Examples
    /// ```
    /// 0b1010
    /// ```
    BinaryLiteral,

    /// Octodecimal literal.
    ///
    /// # Examples
    /// ```
    /// 0o1234
    /// ```
    OctodecimalLiteral,

    /// String literal.
    ///
    /// # Examples
    /// ```
    /// "string"
    /// "string with \"escape\""
    /// "string with line continuation\
    /// "
    /// ```
    StringLiteral,

    /// Char literal.
    ///
    /// # Examples
    /// ```
    /// 'c'
    /// ```
    CharLiteral,

    /// Float literal.
    ///
    /// # Examples
    /// ```
    /// 1.0
    /// 1.0e10
    /// 1.0e-10
    /// ```
    FloatLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comment {
    /// Line comments.
    ///
    /// # Examples
    ///
    /// ```
    /// // comment
    /// ```
    LineComment,

    /// Block comments.
    ///
    /// # Examples
    /// ```
    /// /* comment */
    /// ```
    BlockComment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operator {
    /// `...`
    Ellipses,

    /// `+`
    Plus,

    /// `-`
    Minus,

    /// `*`
    Star,

    /// `/`
    Slash,

    /// `%`
    Percent,

    /// `&`
    Ampersand,

    /// `|`
    Bitor,

    /// `^`
    Bitxor,

    /// `>>`
    Shr,

    /// `>>>`
    Ushr,

    /// `<<`
    Shl,

    /// `=`
    Assign,

    /// `+=`
    AssignAdd,

    /// `-=`
    AssignSub,

    /// `*=`
    AssignMul,

    /// `/=`
    AssignDiv,

    /// `%=`
    AssignMod,

    /// `&=`
    AssignBitAnd,

    /// `|=`
    AssignBitOr,

    /// `^=`
    AssignBitXor,

    /// `>>=`
    AssignShr,

    /// `>>>=`
    AssignUshl,

    /// `<<=`
    AssignShl,

    /// `++`
    Increment,

    /// `--`
    Decrement,

    /// `==`
    Equals,

    /// `!=`
    NotEquals,

    /// `<`
    Lt,

    /// `<=`
    Le,

    /// `>`
    Gt,

    /// `>=`
    Ge,

    /// `&&`
    And,

    /// `||`
    Or,

    /// `!`
    Not,

    /// `~`
    Tilde,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreprocDir {
    /// `#define`
    MDefine,

    /// `#deprecate`
    MDeprecate,

    /// `#else`
    MElse,

    /// `#elseif`
    MEndif,

    /// `#endinput`
    MEndinput,

    /// `#file`
    MFile,

    /// `#if`
    MIf,

    /// `#include`
    MInclude,

    /// `#leaving`
    MLeaving,

    /// `__LINE__`
    MLine,

    /// `#optional_newdecls`
    MOptionalNewdecls,

    /// `#optional_semicolons`
    MOptionalSemi,

    /// `#pragma`
    MPragma,

    /// `#require_newdecls`
    MRequireNewdecls,

    /// `#require_semicolons`
    MRequireSemi,

    /// `#try_include`
    MTryinclude,

    /// `#undef`
    MUndef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
    Literal(Literal),
    Comment(Comment),
    Operator(Operator),
    PreprocDir(PreprocDir),
    Newline,
    LineContinuation,
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
    Intrinsics,
    Semicolon,
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Qmark,
    Colon,
    Scope,
    Dot,

    /// End of file. This will always be the last token.
    Eof,
}

impl TryFrom<Token> for TokenKind {
    type Error = &'static str;

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        let token_kind = match token {
            Token::Identifier => TokenKind::Identifier,
            Token::IntegerLiteral => TokenKind::Literal(Literal::IntegerLiteral),
            Token::HexLiteral => TokenKind::Literal(Literal::HexLiteral),
            Token::BinaryLiteral => TokenKind::Literal(Literal::BinaryLiteral),
            Token::OctodecimalLiteral => TokenKind::Literal(Literal::OctodecimalLiteral),
            Token::StringLiteral => TokenKind::Literal(Literal::StringLiteral),
            Token::CharLiteral => TokenKind::Literal(Literal::CharLiteral),
            Token::FloatLiteral => TokenKind::Literal(Literal::FloatLiteral),
            Token::Newline => TokenKind::Newline,
            Token::LineContinuation => TokenKind::LineContinuation,
            Token::LineComment => TokenKind::Comment(Comment::LineComment),
            Token::BlockComment => TokenKind::Comment(Comment::BlockComment),
            Token::Bool => TokenKind::Bool,
            Token::Break => TokenKind::Break,
            Token::Case => TokenKind::Case,
            Token::Char => TokenKind::Char,
            Token::Class => TokenKind::Class,
            Token::Const => TokenKind::Const,
            Token::Continue => TokenKind::Continue,
            Token::Decl => TokenKind::Decl,
            Token::Default => TokenKind::Default,
            Token::Defined => TokenKind::Defined,
            Token::Delete => TokenKind::Delete,
            Token::Do => TokenKind::Do,
            Token::Else => TokenKind::Else,
            Token::Enum => TokenKind::Enum,
            Token::False => TokenKind::False,
            Token::Float => TokenKind::Float,
            Token::For => TokenKind::For,
            Token::Forward => TokenKind::Forward,
            Token::Functag => TokenKind::Functag,
            Token::Function => TokenKind::Function,
            Token::If => TokenKind::If,
            Token::Int => TokenKind::Int,
            Token::InvalidFunction => TokenKind::InvalidFunction,
            Token::Methodmap => TokenKind::Methodmap,
            Token::Native => TokenKind::Native,
            Token::Null => TokenKind::Null,
            Token::New => TokenKind::New,
            Token::Object => TokenKind::Object,
            Token::Property => TokenKind::Property,
            Token::Public => TokenKind::Public,
            Token::Return => TokenKind::Return,
            Token::Sizeof => TokenKind::Sizeof,
            Token::Static => TokenKind::Static,
            Token::Stock => TokenKind::Stock,
            Token::Struct => TokenKind::Struct,
            Token::Switch => TokenKind::Switch,
            Token::This => TokenKind::This,
            Token::True => TokenKind::True,
            Token::Typedef => TokenKind::Typedef,
            Token::Typeset => TokenKind::Typeset,
            Token::Union => TokenKind::Union,
            Token::Using => TokenKind::Using,
            Token::ViewAs => TokenKind::ViewAs,
            Token::Void => TokenKind::Void,
            Token::While => TokenKind::While,
            Token::Nullable => TokenKind::Nullable,
            Token::MDefine => TokenKind::PreprocDir(PreprocDir::MDefine),
            Token::MDeprecate => TokenKind::PreprocDir(PreprocDir::MDeprecate),
            Token::MElse => TokenKind::PreprocDir(PreprocDir::MElse),
            Token::MEndif => TokenKind::PreprocDir(PreprocDir::MEndif),
            Token::MEndinput => TokenKind::PreprocDir(PreprocDir::MEndinput),
            Token::MFile => TokenKind::PreprocDir(PreprocDir::MFile),
            Token::MIf => TokenKind::PreprocDir(PreprocDir::MIf),
            Token::MInclude => TokenKind::PreprocDir(PreprocDir::MInclude),
            Token::MLeaving => TokenKind::PreprocDir(PreprocDir::MLeaving),
            Token::MLine => TokenKind::PreprocDir(PreprocDir::MLine),
            Token::MOptionalNewdecls => TokenKind::PreprocDir(PreprocDir::MOptionalNewdecls),
            Token::MOptionalSemi => TokenKind::PreprocDir(PreprocDir::MOptionalSemi),
            Token::MPragma => TokenKind::PreprocDir(PreprocDir::MPragma),
            Token::MRequireNewdecls => TokenKind::PreprocDir(PreprocDir::MRequireNewdecls),
            Token::MRequireSemi => TokenKind::PreprocDir(PreprocDir::MRequireSemi),
            Token::MTryinclude => TokenKind::PreprocDir(PreprocDir::MTryinclude),
            Token::MUndef => TokenKind::PreprocDir(PreprocDir::MUndef),
            Token::Intrinsics => TokenKind::Intrinsics,
            Token::Ellipses => TokenKind::Operator(Operator::Ellipses),
            Token::Plus => TokenKind::Operator(Operator::Plus),
            Token::Minus => TokenKind::Operator(Operator::Minus),
            Token::Star => TokenKind::Operator(Operator::Star),
            Token::Slash => TokenKind::Operator(Operator::Slash),
            Token::Percent => TokenKind::Operator(Operator::Percent),
            Token::Ampersand => TokenKind::Operator(Operator::Ampersand),
            Token::Bitor => TokenKind::Operator(Operator::Bitor),
            Token::Bitxor => TokenKind::Operator(Operator::Bitxor),
            Token::Shr => TokenKind::Operator(Operator::Shr),
            Token::Ushr => TokenKind::Operator(Operator::Ushr),
            Token::Shl => TokenKind::Operator(Operator::Shl),
            Token::Assign => TokenKind::Operator(Operator::Assign),
            Token::Semicolon => TokenKind::Semicolon,
            Token::LBrace => TokenKind::LBrace,
            Token::RBrace => TokenKind::RBrace,
            Token::LParen => TokenKind::LParen,
            Token::RParen => TokenKind::RParen,
            Token::LBracket => TokenKind::LBracket,
            Token::RBracket => TokenKind::RBracket,
            Token::AssignAdd => TokenKind::Operator(Operator::AssignAdd),
            Token::AssignSub => TokenKind::Operator(Operator::AssignSub),
            Token::AssignMul => TokenKind::Operator(Operator::AssignMul),
            Token::AssignDiv => TokenKind::Operator(Operator::AssignDiv),
            Token::AssignMod => TokenKind::Operator(Operator::AssignMod),
            Token::AssignBitAnd => TokenKind::Operator(Operator::AssignBitAnd),
            Token::AssignBitOr => TokenKind::Operator(Operator::AssignBitOr),
            Token::AssignBitXor => TokenKind::Operator(Operator::AssignBitXor),
            Token::AssignShr => TokenKind::Operator(Operator::AssignShr),
            Token::AssignUshl => TokenKind::Operator(Operator::AssignUshl),
            Token::AssignShl => TokenKind::Operator(Operator::AssignShl),
            Token::Increment => TokenKind::Operator(Operator::Increment),
            Token::Decrement => TokenKind::Operator(Operator::Decrement),
            Token::Equals => TokenKind::Operator(Operator::Equals),
            Token::NotEquals => TokenKind::Operator(Operator::NotEquals),
            Token::Lt => TokenKind::Operator(Operator::Lt),
            Token::Le => TokenKind::Operator(Operator::Le),
            Token::Gt => TokenKind::Operator(Operator::Gt),
            Token::Ge => TokenKind::Operator(Operator::Ge),
            Token::And => TokenKind::Operator(Operator::And),
            Token::Or => TokenKind::Operator(Operator::Or),
            Token::Comma => TokenKind::Comma,
            Token::Not => TokenKind::Operator(Operator::Not),
            Token::Tilde => TokenKind::Operator(Operator::Tilde),
            Token::Qmark => TokenKind::Qmark,
            Token::Colon => TokenKind::Colon,
            Token::Scope => TokenKind::Scope,
            Token::Dot => TokenKind::Dot,
            _ => return Err("Cannot convert token."),
        };

        Ok(token_kind)
    }
}
