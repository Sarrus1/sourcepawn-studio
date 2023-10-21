#![allow(bad_style, missing_docs, unreachable_pub)]

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u16)]
pub enum SyntaxKind {
    SOURCE_FILE,

    /// (
    LPAREN,

    /// )
    RPAREN,

    /// {
    LBRACE,

    /// }
    RBRACE,

    /// [
    LBRACK,

    /// ]
    RBRACK,

    /// ;
    SEMI,

    /// :
    COLON,

    /// ,
    COMMA,

    /// _
    UNDERSCORE,

    /// .
    DOT,

    /// !
    BANG,

    /// ~
    TILDE,

    /// -
    DASH,

    /// +
    PLUS,

    /// *
    STAR,

    /// /
    SLASH,

    /// =
    EQ,

    /// &
    AMP,

    /// |
    PIPE,

    /// ^
    CARET,

    /// %
    PERCENT,

    /// ?
    QMARK,

    /// >
    GT,

    /// <
    LT,

    /// ||
    PIPE_PIPE,

    /// &&
    AMP_AMP,

    /// ==
    EQ_EQ,

    /// !=
    BANG_EQ,

    /// >=
    GT_EQ,

    /// <=
    LT_EQ,

    /// <<
    LT_LT,

    /// >>
    GT_GT,

    /// ::
    COLON_COLON,

    /// --
    DASH_DASH,

    /// ++
    PLUS_PLUS,

    /// +=
    PLUS_EQ,

    /// -=
    DASH_EQ,

    /// *=
    STAR_EQ,

    /// /=
    SLASH_EQ,

    /// |=
    PIPE_EQ,

    /// &=
    AMP_EQ,

    /// ^=
    CARET_EQ,

    /// ~=
    TILDE_EQ,

    /// <<=
    LT_LT_EQ,

    /// >>=
    GT_GT_EQ,

    /// ...
    REST,

    /// >>>
    GT_GT_GT,

    /// public
    PUBLIC_KW,

    /// stock
    STOCK_KW,

    /// static
    STATIC_KW,

    /// forward
    FORWARD_KW,

    /// native
    NATIVE_KW,

    /// const
    CONST_KW,

    /// null
    NULL_KW,

    /// this
    THIS_KW,

    /// operator
    OPERATOR_KW,

    /// new
    NEW_KW,

    /// decl
    DECL_KW,

    /// true
    TRUE_KW,

    /// false
    FALSE_KW,

    /// enum
    ENUM_KW,

    /// struct
    STRUCT_KW,

    /// typedef
    TYPEDEF_KW,

    /// typeset
    TYPESET_KW,

    /// funcenum
    FUNCENUM_KW,

    /// functag
    FUNCTAG_KW,

    /// methodmap
    METHODMAP_KW,

    /// __nullable__
    NULLABLE_KW,

    /// property
    PROPERTY_KW,

    /// get
    GET_KW,

    /// set
    SET_KW,

    /// any
    ANY_KW,

    /// void
    VOID_KW,

    /// bool
    BOOL_KW,

    /// int
    INT_KW,

    /// float
    FLOAT_KW,

    /// char
    CHAR_KW,

    /// Float
    OLD_FLOAT_KW,

    /// String
    OLD_STRING_KW,

    /// for
    FOR_KW,

    /// while
    WHILE_KW,

    /// do
    DO_KW,

    /// break
    BREAK_KW,

    /// continue
    CONTINUE_KW,

    /// if
    IF_KW,

    /// else
    ELSE_KW,

    /// function
    FUNCTION_KW,

    /// switch
    SWITCH_KW,

    /// case
    CASE_KW,

    /// default
    DEFAULT_KW,

    /// return
    RETURN_KW,

    /// delete
    DELETE_KW,

    /// sizeof
    SIZEOF_KW,

    /// view_as
    VIEW_AS_KW,

    /// #include
    POUNDINCLUDE,

    /// #tryinclude
    POUNDTRYINCLUDE,

    /// #define
    POUNDDEFINE,

    /// #undef
    POUNDUNDEF,

    /// #if
    POUNDIF,

    /// #else
    POUNDELSE,

    /// #elseif
    POUNDELSEIF,

    /// #endif
    POUNDENDIF,

    /// #endinput
    POUNDENDINPUT,

    /// #assert
    POUNDASSERT,

    /// defined
    DEFINED,

    /// #pragma
    POUNDPRAGMA,

    /// #error
    POUNDERROR,

    /// #warning
    POUNDWARNING,

    /// using __intrinsics__.Handle
    USING__INTRINSICS__DOTHANDLE,

    /// assert
    ASSERT,

    /// static_assert
    STATIC_ASSERT,

    MANUAL_SEMICOLON,

    SEMICOLON,

    PREPROC_EXPRESSION,

    PREPROC_PARENTHESIZED_EXPRESSION,

    PREPROC_UNARY_EXPRESSION,

    PREPROC_BINARY_EXPRESSION,

    PREPROC_INCLUDE,

    PREPROC_TRYINCLUDE,

    PREPROC_MACRO,

    MACRO_PARAM,

    PREPROC_DEFINE,

    PREPROC_UNDEFINE,

    PREPROC_IF,

    PREPROC_ELSEIF,

    PREPROC_ASSERT,

    PREPROC_DEFINED_CONDITION,

    PREPROC_ELSE,

    PREPROC_ENDIF,

    PREPROC_ENDINPUT,

    PREPROC_PRAGMA,

    PREPROC_ERROR,

    PREPROC_WARNING,

    HARDCODED_SYMBOL,

    ASSERTION,

    FUNCTION_DECLARATION,

    FUNCTION_VISIBILITY,

    FUNCTION_DEFINITION,

    FUNCTION_DEFINITION_TYPE,

    ARGUMENT_DECLARATIONS,

    ARGUMENT_TYPE,

    ARGUMENT_DECLARATION,

    REST_ARGUMENT,

    ALIAS_OPERATOR,

    ALIAS_DECLARATION,

    ALIAS_ASSIGNMENT,

    GLOBAL_VARIABLE_DECLARATION,

    VARIABLE_DECLARATION_STATEMENT,

    VARIABLE_STORAGE_CLASS,

    VARIABLE_VISIBILITY,

    VARIABLE_DECLARATION,

    DYNAMIC_ARRAY,

    NEW_INSTANCE,

    OLD_GLOBAL_VARIABLE_DECLARATION,

    OLD_VARIABLE_DECLARATION_STATEMENT,

    OLD_VARIABLE_DECLARATION,

    ENUM,

    ENUM_ENTRIES,

    ENUM_ENTRY,

    ENUM_STRUCT,

    ENUM_STRUCT_FIELD,

    ENUM_STRUCT_METHOD,

    TYPEDEF,

    TYPESET,

    TYPEDEF_EXPRESSION,

    FUNCENUM,

    FUNCENUM_MEMBER,

    FUNCTAG,

    METHODMAP,

    METHODMAP_ALIAS,

    METHODMAP_NATIVE,

    METHODMAP_NATIVE_CONSTRUCTOR,

    METHODMAP_NATIVE_DESTRUCTOR,

    METHODMAP_METHOD,

    METHODMAP_METHOD_CONSTRUCTOR,

    METHODMAP_METHOD_DESTRUCTOR,

    METHODMAP_PROPERTY,

    METHODMAP_PROPERTY_ALIAS,

    METHODMAP_PROPERTY_NATIVE,

    METHODMAP_PROPERTY_METHOD,

    METHODMAP_PROPERTY_GETTER,

    METHODMAP_PROPERTY_SETTER,

    METHODMAP_VISIBILITY,

    STRUCT,

    STRUCT_FIELD,

    STRUCT_DECLARATION,

    STRUCT_CONSTRUCTOR,

    STRUCT_FIELD_VALUE,

    TYPE,

    OLD_TYPE,

    DIMENSION,

    FIXED_DIMENSION,

    BUILTIN_TYPE,

    OLD_BUILTIN_TYPE,

    BLOCK,

    STATEMENT,

    FOR_STATEMENT,

    WHILE_STATEMENT,

    DO_WHILE_STATEMENT,

    BREAK_STATEMENT,

    CONTINUE_STATEMENT,

    CONDITION_STATEMENT,

    SWITCH_STATEMENT,

    SWITCH_CASE,

    SWITCH_CASE_VALUES,

    SWITCH_DEFAULT_CASE,

    EXPRESSION_STATEMENT,

    RETURN_STATEMENT,

    DELETE_STATEMENT,

    EXPRESSION,

    ASSIGNMENT_EXPRESSION,

    FUNCTION_CALL,

    FUNCTION_CALL_ARGUMENTS,

    NAMED_ARG,

    ARRAY_INDEXED_ACCESS,

    PARENTHESIZED_EXPRESSION,

    COMMA_EXPRESSION,

    TERNARY_EXPRESSION,

    FIELD_ACCESS,

    SCOPE_ACCESS,

    UNARY_EXPRESSION,

    BINARY_EXPRESSION,

    UPDATE_EXPRESSION,

    SIZEOF_CALL_EXPRESSION,

    ARRAY_SCOPE_ACCESS,

    SIZEOF,

    VIEW_AS,

    OLD_TYPE_CAST,

    ARRAY_LITERAL,

    LITERAL,

    INT_LITERAL,

    FLOAT_LITERAL,

    CHAR_LITERAL,

    CONCATENATED_STRING,

    STRING_LITERAL,

    ESCAPE_SEQUENCE,

    SYSTEM_LIB_STRING,

    SYMBOL,

    COMMENT,

    #[doc(hidden)]
    __LAST,
}
#[macro_export]
macro_rules ! T {['('] => { $ crate::SyntaxKind::LPAREN } ; [')'] => { $ crate::SyntaxKind::RPAREN } ; ['{'] => { $ crate::SyntaxKind::LBRACE } ; ['}'] => { $ crate::SyntaxKind::RBRACE } ; ['['] => { $ crate::SyntaxKind::LBRACK } ; [']'] => { $ crate::SyntaxKind::RBRACK } ; [;] => { $ crate::SyntaxKind::SEMI } ; [:] => { $ crate::SyntaxKind::COLON } ; [,] => { $ crate::SyntaxKind::COMMA } ; [_] => { $ crate::SyntaxKind::UNDERSCORE } ; [.] => { $ crate::SyntaxKind::DOT } ; [!] => { $ crate::SyntaxKind::BANG } ; [~] => { $ crate::SyntaxKind::TILDE } ; [-] => { $ crate::SyntaxKind::DASH } ; [+] => { $ crate::SyntaxKind::PLUS } ; [*] => { $ crate::SyntaxKind::STAR } ; [/] => { $ crate::SyntaxKind::SLASH } ; [=] => { $ crate::SyntaxKind::EQ } ; [&] => { $ crate::SyntaxKind::AMP } ; [|] => { $ crate::SyntaxKind::PIPE } ; [^] => { $ crate::SyntaxKind::CARET } ; [%] => { $ crate::SyntaxKind::PERCENT } ; [?] => { $ crate::SyntaxKind::QMARK } ; [>] => { $ crate::SyntaxKind::GT } ; [<] => { $ crate::SyntaxKind::LT } ; [||] => { $ crate::SyntaxKind::PIPE_PIPE } ; [&&] => { $ crate::SyntaxKind::AMP_AMP } ; [==] => { $ crate::SyntaxKind::EQ_EQ } ; [!=] => { $ crate::SyntaxKind::BANG_EQ } ; [>=] => { $ crate::SyntaxKind::GT_EQ } ; [<=] => { $ crate::SyntaxKind::LT_EQ } ; [<<] => { $ crate::SyntaxKind::LT_LT } ; [>>] => { $ crate::SyntaxKind::GT_GT } ; [::] => { $ crate::SyntaxKind::COLON_COLON } ; [--] => { $ crate::SyntaxKind::DASH_DASH } ; [++] => { $ crate::SyntaxKind::PLUS_PLUS } ; [+=] => { $ crate::SyntaxKind::PLUS_EQ } ; [-=] => { $ crate::SyntaxKind::DASH_EQ } ; [*=] => { $ crate::SyntaxKind::STAR_EQ } ; [/=] => { $ crate::SyntaxKind::SLASH_EQ } ; [|=] => { $ crate::SyntaxKind::PIPE_EQ } ; [&=] => { $ crate::SyntaxKind::AMP_EQ } ; [^=] => { $ crate::SyntaxKind::CARET_EQ } ; [~=] => { $ crate::SyntaxKind::TILDE_EQ } ; [<<=] => { $ crate::SyntaxKind::LT_LT_EQ } ; [>>=] => { $ crate::SyntaxKind::GT_GT_EQ } ; [...] => { $ crate::SyntaxKind::REST } ; [>>>] => { $ crate::SyntaxKind::GT_GT_GT } ; [public] => { $ crate::SyntaxKind::PUBLIC_KW } ; [stock] => { $ crate::SyntaxKind::STOCK_KW } ; [static] => { $ crate::SyntaxKind::STATIC_KW } ; [forward] => { $ crate::SyntaxKind::FORWARD_KW } ; [native] => { $ crate::SyntaxKind::NATIVE_KW } ; [const] => { $ crate::SyntaxKind::CONST_KW } ; [null] => { $ crate::SyntaxKind::NULL_KW } ; [this] => { $ crate::SyntaxKind::THIS_KW } ; [operator] => { $ crate::SyntaxKind::OPERATOR_KW } ; [new] => { $ crate::SyntaxKind::NEW_KW } ; [decl] => { $ crate::SyntaxKind::DECL_KW } ; [true] => { $ crate::SyntaxKind::TRUE_KW } ; [false] => { $ crate::SyntaxKind::FALSE_KW } ; [enum] => { $ crate::SyntaxKind::ENUM_KW } ; [struct] => { $ crate::SyntaxKind::STRUCT_KW } ; [typedef] => { $ crate::SyntaxKind::TYPEDEF_KW } ; [typeset] => { $ crate::SyntaxKind::TYPESET_KW } ; [funcenum] => { $ crate::SyntaxKind::FUNCENUM_KW } ; [functag] => { $ crate::SyntaxKind::FUNCTAG_KW } ; [methodmap] => { $ crate::SyntaxKind::METHODMAP_KW } ; [__nullable__] => { $ crate::SyntaxKind::NULLABLE_KW } ; [property] => { $ crate::SyntaxKind::PROPERTY_KW } ; [get] => { $ crate::SyntaxKind::GET_KW } ; [set] => { $ crate::SyntaxKind::SET_KW } ; [any] => { $ crate::SyntaxKind::ANY_KW } ; [void] => { $ crate::SyntaxKind::VOID_KW } ; [bool] => { $ crate::SyntaxKind::BOOL_KW } ; [int] => { $ crate::SyntaxKind::INT_KW } ; [float] => { $ crate::SyntaxKind::FLOAT_KW } ; [char] => { $ crate::SyntaxKind::CHAR_KW } ; [Float] => { $ crate::SyntaxKind::OLD_FLOAT_KW } ; [String] => { $ crate::SyntaxKind::OLD_STRING_KW } ; [for] => { $ crate::SyntaxKind::FOR_KW } ; [while] => { $ crate::SyntaxKind::WHILE_KW } ; [do] => { $ crate::SyntaxKind::DO_KW } ; [break] => { $ crate::SyntaxKind::BREAK_KW } ; [continue] => { $ crate::SyntaxKind::CONTINUE_KW } ; [if] => { $ crate::SyntaxKind::IF_KW } ; [else] => { $ crate::SyntaxKind::ELSE_KW } ; [function] => { $ crate::SyntaxKind::FUNCTION_KW } ; [switch] => { $ crate::SyntaxKind::SWITCH_KW } ; [case] => { $ crate::SyntaxKind::CASE_KW } ; [default] => { $ crate::SyntaxKind::DEFAULT_KW } ; [return] => { $ crate::SyntaxKind::RETURN_KW } ; [delete] => { $ crate::SyntaxKind::DELETE_KW } ; [sizeof] => { $ crate::SyntaxKind::SIZEOF_KW } ; [view_as] => { $ crate::SyntaxKind::VIEW_AS_KW } ; [#include] => { $ crate::SyntaxKind::POUNDINCLUDE } ; [#tryinclude] => { $ crate::SyntaxKind::POUNDTRYINCLUDE } ; [#define] => { $ crate::SyntaxKind::POUNDDEFINE } ; [#undef] => { $ crate::SyntaxKind::POUNDUNDEF } ; [#if] => { $ crate::SyntaxKind::POUNDIF } ; [#else] => { $ crate::SyntaxKind::POUNDELSE } ; [#elseif] => { $ crate::SyntaxKind::POUNDELSEIF } ; [#endif] => { $ crate::SyntaxKind::POUNDENDIF } ; [#endinput] => { $ crate::SyntaxKind::POUNDENDINPUT } ; [#assert] => { $ crate::SyntaxKind::POUNDASSERT } ; [defined] => { $ crate::SyntaxKind::DEFINED } ; [#pragma] => { $ crate::SyntaxKind::POUNDPRAGMA } ; [#error] => { $ crate::SyntaxKind::POUNDERROR } ; [#warning] => { $ crate::SyntaxKind::POUNDWARNING } ; [using __intrinsics__.Handle] => { $ crate::SyntaxKind::USING__INTRINSICS__DOTHANDLE } ; [assert] => { $ crate::SyntaxKind::ASSERT } ; [static_assert] => { $ crate::SyntaxKind::STATIC_ASSERT }}
pub use T;
