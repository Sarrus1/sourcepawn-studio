use crate::pragma::lex_pragma_arguments;
use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq, Eq)]
// white space
#[logos(subpattern ws = r"[ \t\v\f]")]
// escape sequence
#[logos(subpattern es = r#"[\\](['"%?\\abefnrtv]|[0-7]+|[xu][a-fA-F0-9]+|[\r]?[\n])"#)]
pub enum Token {
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[regex(r"\d[0-9_]*")]
    IntegerLiteral,

    #[regex(r"0x[0-9a-fA-F_]+")]
    HexLiteral,

    #[regex(r"0b[01_]+")]
    BinaryLiteral,

    #[regex(r"0o[0-7_]+")]
    OctodecimalLiteral,

    #[regex(r#""([^"\\\n]|(?&es))*""#)]
    #[regex(r#"<([^>\\\n]|(?&es))*>"#)]
    StringLiteral,

    #[regex(r"'([^'\\\n]|(?&es))*'")]
    CharLiteral,

    #[regex(r"(?:(?:[0-9_]+\.[0-9_]*)|(?:[0-9_]+\.[0-9_]+)|(?:[0-9_]*\.[0-9_]+))(e\-?\d+)?")]
    FloatLiteral,

    #[regex(r"\r?\n")]
    Newline,

    #[regex(r"\\\r?\n")]
    LineContinuation,

    #[regex("//[^\r\n]*")]
    LineComment,

    #[token("/*",
        |lex| {
        lex.bump(lex.remainder().find("*/")? + 2);
        Some(())
    })]
    BlockComment,

    #[token("bool")]
    Bool,

    #[token("break")]
    Break,

    #[token("case")]
    Case,

    #[token("char")]
    Char,

    #[token("class")]
    Class,

    #[token("const")]
    Const,

    #[token("continue")]
    Continue,

    #[token("decl")]
    Decl,

    #[token("default")]
    Default,

    #[token("defined")]
    Defined,

    #[token("delete")]
    Delete,

    #[token("do")]
    Do,

    #[token("else")]
    Else,

    #[token("enum")]
    Enum,

    #[token("false")]
    False,

    #[token("float")]
    Float,

    #[token("for")]
    For,

    #[token("forward")]
    Forward,

    #[token("functag")]
    Functag,

    #[token("function")]
    Function,

    #[token("if")]
    If,

    #[token("int")]
    Int,

    #[token("INVALID_FUNCTION")]
    InvalidFunction,

    #[token("methodmap")]
    Methodmap,

    #[token("native")]
    Native,

    #[token("null")]
    Null,

    #[token("new")]
    New,

    #[token("object")]
    Object,

    #[token("property")]
    Property,

    #[token("public")]
    Public,

    #[token("return")]
    Return,

    #[token("sizeof")]
    Sizeof,

    #[token("static")]
    Static,

    #[token("stock")]
    Stock,

    #[token("struct")]
    Struct,

    #[token("switch")]
    Switch,

    #[token("this")]
    This,

    #[token("true")]
    True,

    #[token("typedef")]
    Typedef,

    #[token("typeset")]
    Typeset,

    #[token("union")]
    Union,

    #[token("using")]
    Using,

    #[token("view_as")]
    ViewAs,

    #[token("void")]
    Void,

    #[token("while")]
    While,

    #[token("__nullable__")]
    Nullable,

    #[token("#define")]
    MDefine,

    #[token("#deprecate")]
    MDeprecate,

    #[token("#else")]
    MElse,

    #[token("#endif")]
    MEndif,

    #[token("#endinput")]
    MEndinput,

    #[token("#file")]
    MFile,

    #[token("#if")]
    MIf,

    // TODO: Handle include strings
    #[token("#include")]
    MInclude,

    #[token("#leaving")]
    MLeaving,

    #[token("__LINE__")]
    MLine,

    #[token("#optional_newdecls")]
    MOptionalNewdecls,

    #[token("#optional_semicolons")]
    MOptionalSemi,

    #[token("#pragma", lex_pragma_arguments)]
    MPragma,

    #[token("#require_newdecls")]
    MRequireNewdecls,

    #[token("#require_semicolons")]
    MRequireSemi,

    #[token("#tryinclude")]
    MTryinclude,

    #[token("#undef")]
    MUndef,

    #[token("__intrinsics__")]
    Intrinsics,

    #[token("...")]
    Ellipses,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("%")]
    Percent,

    #[token("&")]
    Ampersand,

    #[token("|")]
    Bitor,

    #[token("^")]
    Bitxor,

    #[token(">>")]
    Shr,

    #[token(">>>")]
    Ushr,

    #[token("<<")]
    Shl,

    #[token("=")]
    Assign,

    #[token(";")]
    Semicolon,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token("+=")]
    AssignAdd,

    #[token("-=")]
    AssignSub,

    #[token("*=")]
    AssignMul,

    #[token("/=")]
    AssignDiv,

    #[token("%=")]
    AssignMod,

    #[token("&=")]
    AssignBitAnd,

    #[token("|=")]
    AssignBitOr,

    #[token("^=")]
    AssignBitXor,

    #[token(">>=")]
    AssignShr,

    #[token(">>>=")]
    AssignUshl,

    #[token("<<=")]
    AssignShl,

    #[token("++")]
    Increment,

    #[token("--")]
    Decrement,

    #[token("==")]
    Equals,

    #[token("!=")]
    NotEquals,

    #[token("<")]
    Lt,

    #[token("<=")]
    Le,

    #[token(">")]
    Gt,

    #[token(">=")]
    Ge,

    #[token("&&")]
    And,

    #[token("||")]
    Or,

    #[token(",")]
    Comma,

    #[token("!")]
    Not,

    #[token("~")]
    Tilde,

    #[token("?")]
    Qmark,

    #[token(":")]
    Colon,

    #[token("::")]
    Scope,

    #[token(".")]
    Dot,

    #[error]
    #[regex(r"(?&ws)+", logos::skip)]
    Unknown,
}
