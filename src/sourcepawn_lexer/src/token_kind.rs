use crate::token::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
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
}

impl TryFrom<Token> for TokenKind {
    type Error = &'static str;

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        let token_kind = match token {
            Token::Identifier => TokenKind::Identifier,
            Token::IntegerLiteral => TokenKind::IntegerLiteral,
            Token::HexLiteral => TokenKind::HexLiteral,
            Token::BinaryLiteral => TokenKind::BinaryLiteral,
            Token::OctodecimalLiteral => TokenKind::OctodecimalLiteral,
            Token::StringLiteral => TokenKind::StringLiteral,
            Token::CharLiteral => TokenKind::CharLiteral,
            Token::FloatLiteral => TokenKind::FloatLiteral,
            Token::Newline => TokenKind::Newline,
            Token::LineContinuation => TokenKind::LineContinuation,
            Token::LineComment => TokenKind::LineComment,
            Token::BlockComment => TokenKind::BlockComment,
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
            Token::MDefine => TokenKind::MDefine,
            Token::MDeprecate => TokenKind::MDeprecate,
            Token::MElse => TokenKind::MElse,
            Token::MEndif => TokenKind::MEndif,
            Token::MEndinput => TokenKind::MEndinput,
            Token::MFile => TokenKind::MFile,
            Token::MIf => TokenKind::MIf,
            Token::MInclude => TokenKind::MInclude,
            Token::MLeaving => TokenKind::MLeaving,
            Token::MLine => TokenKind::MLine,
            Token::MOptionalNewdecls => TokenKind::MOptionalNewdecls,
            Token::MOptionalSemi => TokenKind::MOptionalSemi,
            Token::MPragma => TokenKind::MPragma,
            Token::MRequireNewdecls => TokenKind::MRequireNewdecls,
            Token::MRequireSemi => TokenKind::MRequireSemi,
            Token::MTryinclude => TokenKind::MTryinclude,
            Token::MUndef => TokenKind::MUndef,
            Token::Intrinsics => TokenKind::Intrinsics,
            Token::Ellipses => TokenKind::Ellipses,
            Token::Plus => TokenKind::Plus,
            Token::Minus => TokenKind::Minus,
            Token::Star => TokenKind::Star,
            Token::Slash => TokenKind::Slash,
            Token::Percent => TokenKind::Percent,
            Token::Ampersand => TokenKind::Ampersand,
            Token::Bitor => TokenKind::Bitor,
            Token::Bitxor => TokenKind::Bitxor,
            Token::Shr => TokenKind::Shr,
            Token::Ushr => TokenKind::Ushr,
            Token::Shl => TokenKind::Shl,
            Token::Assign => TokenKind::Assign,
            Token::Semicolon => TokenKind::Semicolon,
            Token::LBrace => TokenKind::LBrace,
            Token::RBrace => TokenKind::RBrace,
            Token::LParen => TokenKind::LParen,
            Token::RParen => TokenKind::RParen,
            Token::LBracket => TokenKind::LBracket,
            Token::RBracket => TokenKind::RBracket,
            Token::AssignAdd => TokenKind::AssignAdd,
            Token::AssignSub => TokenKind::AssignSub,
            Token::AssignMul => TokenKind::AssignMul,
            Token::AssignDiv => TokenKind::AssignDiv,
            Token::AssignMod => TokenKind::AssignMod,
            Token::AssignBitAnd => TokenKind::AssignBitAnd,
            Token::AssignBitOr => TokenKind::AssignBitOr,
            Token::AssignBitXor => TokenKind::AssignBitXor,
            Token::AssignShr => TokenKind::AssignShr,
            Token::AssignUshl => TokenKind::AssignUshl,
            Token::AssignShl => TokenKind::AssignShl,
            Token::Increment => TokenKind::Increment,
            Token::Decrement => TokenKind::Decrement,
            Token::Equals => TokenKind::Equals,
            Token::NotEquals => TokenKind::NotEquals,
            Token::Lt => TokenKind::Lt,
            Token::Le => TokenKind::Le,
            Token::Gt => TokenKind::Gt,
            Token::Ge => TokenKind::Ge,
            Token::And => TokenKind::And,
            Token::Or => TokenKind::Or,
            Token::Comma => TokenKind::Comma,
            Token::Not => TokenKind::Not,
            Token::Tilde => TokenKind::Tilde,
            Token::Qmark => TokenKind::Qmark,
            Token::Colon => TokenKind::Colon,
            Token::Scope => TokenKind::Scope,
            Token::Dot => TokenKind::Dot,
            _ => return Err("Cannot convert token."),
        };

        Ok(token_kind)
    }
}
