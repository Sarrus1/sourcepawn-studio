#![allow(bad_style, missing_docs, unreachable_pub)]

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u16)]
pub enum SyntaxKind {
    /// ;
    MANUAL_SEMICOLON,
    /// (
    L_PAREN,
    /// )
    R_PAREN,
    /// [
    L_BRACKET,
    /// ]
    R_BRACKET,
    /// {
    L_CURLY,
    /// }
    R_CURLY,
    /// bool
    BOOL_KW,
    /// char
    CHAR_KW,
    /// float
    FLOAT_KW,
    /// int
    INT_KW,
    /// Float
    OLD_FLOAT_KW,
    /// String
    STRING_KW,
    /// break
    BREAK_KW,
    /// case
    CASE_KW,
    /// return
    RETURN_KW,
    /// continue
    CONTINUE_KW,
    /// #else
    PREPROC_ELSE,
    /// #endif
    PREPROC_ENDIF,
    /// #endinput
    PREPROC_ENDINPUT,
    /// public
    METHODMAP_VISIBILITY,
    /// any
    ANY_TYPE,
    /// _
    IGNORE_ARGUMENT,
    /// null
    NULL,
    /// this
    THIS,
    /// ...
    REST_OPERATOR,
}
