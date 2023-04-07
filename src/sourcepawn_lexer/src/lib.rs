use itertools::Itertools;
use logos::{Lexer, Logos};

fn lex_pragma_arguments(lex: &mut Lexer<Token>) -> Option<()> {
    let mut in_block_comment = false;
    let mut looking_for_newline = false;
    let mut ignore_newline = false;
    let mut offset = 0;
    for (ch, next_ch) in lex.remainder().chars().tuple_windows() {
        if in_block_comment {
            match ch {
                '*' => {
                    if next_ch == '/' {
                        // Exit block comment.
                        in_block_comment = false;
                        looking_for_newline = true;
                    }
                }
                '\\' => {
                    if next_ch == '\n' {
                        // Line continuation in block comment.
                        ignore_newline = true;
                    }
                }
                '\n' => {
                    if ignore_newline {
                        // Line continuation.
                        ignore_newline = false;
                    } else {
                        // Newline in block comment breaks the pragma.
                        return Some(());
                    }
                }
                _ => {}
            }
            offset += 1;
        } else if looking_for_newline {
            // Lookahead for a newline without any non-whitespace characters.
            if next_ch == '\n' {
                // Found a newline, the block comment is not part of the pragma.
                return Some(());
            }
            if next_ch.is_whitespace() {
                offset += 1;
            } else {
                // Non-whitespace character found, bump the lexer and continue.
                lex.bump(offset + 2);
                looking_for_newline = false;
            }
        } else {
            match ch {
                '/' => {
                    match next_ch {
                        '/' => {
                            // Line comments break the pragma.
                            return Some(());
                        }
                        '*' => {
                            // Enter block comment.
                            in_block_comment = true;
                            continue;
                        }
                        _ => {}
                    }
                }
                '\n' => {
                    if !ignore_newline {
                        // Reached the end of the pragma.
                        return Some(());
                    }
                    // Line continuation.
                    ignore_newline = false;
                }
                '\\' => {
                    if next_ch == '\n' {
                        // Line continuation.
                        ignore_newline = true;
                    }
                }
                _ => {}
            }
            lex.bump(1);
        }
    }

    Some(())
}

#[derive(Logos, Debug, PartialEq, Eq)]
// char prefix
#[logos(subpattern cp = r"[uUL]")]
// white space
#[logos(subpattern ws = r"[ \t\v\r\n\f]")]
// escape sequence
#[logos(subpattern es = r#"[\\](['"%?\\abefnrtv]|[0-7]+|[xu][a-fA-F0-9]+|[\r]?[\n])"#)]
pub enum Token {
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[regex(r"\d[0-9_]*")]
    IntegerLiteral,

    #[regex(r"0x[0-9a-fA-F]+")]
    HexLiteral,

    #[regex(r"0b[01]+")]
    BinaryLiteral,

    #[regex(r"0o[0-7]+")]
    OctodecimalLiteral,

    #[regex(r#""([^"\\\n]|(?&es))*""#)]
    StringLiteral,

    #[regex(r"(?&cp)?'([^'\\\n]|(?&es))*'")]
    CharLiteral,

    #[regex(r"\d[0-9_]*\.[0-9_]+(e\-?\d+)?")]
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
