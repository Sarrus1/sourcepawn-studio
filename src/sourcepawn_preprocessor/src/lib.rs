pub(crate) mod evaluator;
mod macros;
pub mod preprocessor;
mod preprocessor_operator;

pub use self::preprocessor::SourcepawnPreprocessor;

#[cfg(test)]
mod test {
    use fxhash::FxHashMap;
    use sourcepawn_lexer::{SourcepawnLexer, TokenKind};

    use crate::{evaluator::IfCondition, preprocessor::Macro, SourcepawnPreprocessor};

    #[test]
    fn no_preprocessor_directives() {
        let input = r#"
        int foo;
        int bar;
        "#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), input);
    }

    fn evaluate_if_condition(input: &str) -> bool {
        let mut lexer = SourcepawnLexer::new(input);
        let macros: FxHashMap<String, Macro> = FxHashMap::default();
        let mut if_condition = IfCondition::new(&macros);
        if let Some(symbol) = lexer.next() {
            if TokenKind::PreprocDir(sourcepawn_lexer::PreprocDir::MIf) == symbol.token_kind {
                while lexer.in_preprocessor() {
                    if let Some(symbol) = lexer.next() {
                        if_condition.symbols.push(symbol);
                    } else {
                        break;
                    }
                }
            }
        }

        if_condition.evaluate()
    }

    #[test]
    fn if_directive_simple_true() {
        let input = r#"#if 1"#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_simple_false() {
        let input = r#"#if 0"#;

        assert!(!evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_simple_true_with_ws() {
        let input = r#"#if 1 "#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_simple_true_parenthesis() {
        let input = r#"#if (1)"#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_simple_binary_true() {
        let input = r#"#if 1+1"#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_simple_binary_false() {
        let input = r#"#if 1-1"#;

        assert!(!evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_simple_unary_false() {
        let input = r#"#if !1"#;

        assert!(!evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_equality_true() {
        let input = r#"#if 1 == 1"#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_difference_true() {
        let input = r#"#if 1 != 0"#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_equality_false() {
        let input = r#"#if 1 == 0"#;

        assert!(!evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_difference_false() {
        let input = r#"#if 1 != 1"#;

        assert!(!evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_complex_expression_1() {
        let input = r#"#if (1 + 1) && (0 + 0)"#;

        assert!(!evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_complex_expression_2() {
        let input = r#"#if (true && 1) || (true + 1)"#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_int_1() {
        let input = r#"#if 1000 == 1000"#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_int_2() {
        let input = r#"#if (10_00) == 1000"#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_binary_1() {
        let input = r#"#if 0x0001 == 0x0001"#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_binary_2() {
        let input = r#"#if 0x000_1 == 0x000_1"#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_char_1() {
        let input = r#"#if 'a' == 'a'"#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_char_2() {
        let input = r#"#if 'a' != 'b'"#;

        assert!(evaluate_if_condition(input));
    }

    #[test]
    fn if_directive_defined() {
        let input = r#"#define FOO
#if defined FOO
    int foo;
#endif"#;
        let output = r#"#define FOO

    int foo;
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn if_directive_defined_complex_1() {
        let input = r#"#define FOO
#if defined FOO && defined BAR
    int foo;
    int bar;
#endif"#;
        let output = r#"#define FOO



"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn if_directive_defined_complex_2() {
        let input = r#"#define FOO
#define BAR
#if defined FOO && defined BAR
    int foo;
    int bar;
#endif"#;
        let output = r#"#define FOO
#define BAR

    int foo;
    int bar;
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn if_directive_defined_complex_3() {
        let input = r#"#define FOO
#define BAR
#if defined FOO
    int foo;
    #if defined BAR
    int bar;
    #endif
#endif"#;
        let output = r#"#define FOO
#define BAR

    int foo;

    int bar;

"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn if_directive_defined_complex_4() {
        let input = r#"#define FOO
#if defined FOO
    int foo;
    #if defined BAZ
    int bar;
    #else
    int baz;
    #endif
#endif"#;
        let output = r#"#define FOO

    int foo;



    int baz;

"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn if_directive_expansion_1() {
        let input = r#"#define FOO 1
#if FOO
    int foo;
#endif"#;
        let output = r#"#define FOO 1

    int foo;
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn if_directive_expansion_2() {
        let input = r#"#define FOO 1
#if FOO == 1
    int foo;
#endif"#;
        let output = r#"#define FOO 1

    int foo;
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn if_directive_nested_expansion_1() {
        let input = r#"#define FOO BAR
#define BAR 1
#if FOO == 1
    int foo;
#endif"#;
        let output = r#"#define FOO BAR
#define BAR 1

    int foo;
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn if_directive_nested_expansion_2() {
        let input = r#"#define FOO BAR + 4
#define BAR 1 + BAZ
#define BAZ 2 + 3
#if FOO == 10
    int foo;
#endif"#;
        let output = r#"#define FOO BAR + 4
#define BAR 1 + BAZ
#define BAZ 2 + 3

    int foo;
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn if_directive_nested_expansion_infinite_loop_1() {
        let input = r#"#define FOO BAR
#define BAR FOO
#if FOO == 1
    int foo;
#endif"#;
        let output = r#"#define FOO BAR
#define BAR FOO


"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn define_expansion_1() {
        let input = r#"#define FOO 1
int foo = FOO;
"#;
        let output = r#"#define FOO 1
int foo = 1;
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn define_expansion_2() {
        let input = r#"#define FOO "test"
char foo[64] = FOO;
"#;
        let output = r#"#define FOO "test"
char foo[64] = "test";
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn define_expansion_comment_1() {
        let input = r#"#define FOO 1 //comment
int foo = FOO;
"#;
        let output = r#"#define FOO 1 //comment
int foo = 1;
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn define_expansion_comment_2() {
        let input = r#"#define FOO 1 /* comment */
int foo = FOO;
"#;
        let output = r#"#define FOO 1 /* comment */
int foo = 1;
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn define_expansion_comment_3() {
        let input = r#"#define FOO 1 /* long\
comment */
int foo = FOO;
"#;
        let output = r#"#define FOO 1 /* long\
comment */
int foo = 1;
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn define_expansion_comment_4() {
        let input = r#"#define FOO 1 /* long\
comment */ + 2
int foo = FOO;
int bar;
"#;
        let output = r#"#define FOO 1 /* long\
comment */ + 2
int foo = 1 + 2;
int bar;
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn define_expansion_comment_5() {
        let input = r#"#define FOO 1 /* long\
comment */ + 2 // Line comment
int foo = FOO;
int bar;
"#;
        let output = r#"#define FOO 1 /* long\
comment */ + 2 // Line comment
int foo = 1 + 2;
int bar;
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn define_expansion_line_continuation_1() {
        let input = r#"#define FOO "test \
expansion"
char foo[64] = FOO;
"#;
        let output = r#"#define FOO "test \
expansion"
char foo[64] = "test expansion";
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn define_expansion_line_continuation_2() {
        let input = r#"#define FOO "test \
expansion \
\
also"
char foo[64] = FOO;
"#;
        let output = r#"#define FOO "test \
expansion \
\
also"
char foo[64] = "test expansion also";
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn define_expansion_nested_1() {
        let input = r#"#define FOO BAR +   2
#define BAR 1
int foo = FOO;
"#;
        let output = r#"#define FOO BAR +   2
#define BAR 1
int foo = 1 +   2;
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn define_expansion_nested_2() {
        let input = r#"#define FOO BAR + 3
#define BAR 1 + BAZ
#define BAZ 2
int foo = FOO;
"#;
        let output = r#"#define FOO BAR + 3
#define BAR 1 + BAZ
#define BAZ 2
int foo = 1 + 2 + 3;
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn macro_expansion_1() {
        let input = r#"#define FOO(%0,%1) %0 + %1
int foo = FOO(1, 2);
"#;
        let output = r#"#define FOO(%0,%1) %0 + %1
int foo = 1 + 2;
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn macro_expansion_2() {
        let input = r#"#define FOO(%0) %0 %%2
int foo = FOO(2);
"#;
        let output = r#"#define FOO(%0) %0 %%2
int foo = 2 %2;
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }

    #[test]
    fn macro_expansion_3() {
        let input = r#"#define FOO(%0,%1) %0 + %1
#define BAR(%0,%1) %0 + %1
int foo = FOO(1, BAR(2, 3 + 4));
"#;
        let output = r#"#define FOO(%0,%1) %0 + %1
#define BAR(%0,%1) %0 + %1
int foo = 1 + 2 + 3 + 4;
"#;

        let mut preprocessor = SourcepawnPreprocessor::new(input);
        assert_eq!(preprocessor.preprocess_input(), output);
    }
}
