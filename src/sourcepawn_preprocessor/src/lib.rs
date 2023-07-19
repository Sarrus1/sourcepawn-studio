mod errors;
pub(crate) mod evaluator;
mod macros;
pub mod preprocessor;
mod preprocessor_operator;

pub use self::preprocessor::SourcepawnPreprocessor;

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use fxhash::FxHashMap;
    use lsp_types::Url;
    use sourcepawn_lexer::{SourcepawnLexer, TokenKind};

    fn extend_macros(
        _macros: &mut FxHashMap<String, Macro>,
        mut _path: String,
        _document_uri: &Url,
        _quoted: bool,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    use super::{evaluator::IfCondition, preprocessor::Macro, SourcepawnPreprocessor};

    #[test]
    fn no_preprocessor_directives() {
        let input = r#"
        int foo;
        int bar;
        "#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            input
        );
    }

    fn evaluate_if_condition(input: &str) -> bool {
        let mut lexer = SourcepawnLexer::new(input);
        let macros: FxHashMap<String, Macro> = FxHashMap::default();
        let mut if_condition = IfCondition::new(&macros, 0);
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

        if_condition.evaluate().unwrap_or(false)
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

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
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

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
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

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
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

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
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

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn if_directive_defined_complex_5() {
        let input = r#"#define FOO
#if defined FOO
int foo;
#else
int bar;
#endif"#;
        let output = r#"#define FOO

int foo;


"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn if_directive_defined_complex_6() {
        let input = r#"#define FOO
#if defined BAR
int foo;
#else
int bar;
#endif"#;
        let output = r#"#define FOO



int bar;
"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn if_directive_defined_complex_7() {
        let input = r#"#define FOO
#if defined FOO
public void OnPluginStart()
#else
#if 1==1
public void OnPluginStart(int args)
#else
int bar;
#endif
#endif
{
    int bar;
}"#;
        let output = r#"#define FOO

public void OnPluginStart()







{
    int bar;
}"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn if_directive_defined_complex_8() {
        let input = r#"#define FOO 1
#if defined FOO
int foo;
#elseif FOO == 1
int bar;
#endif"#;
        let output = r#"#define FOO 1

int foo;


"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn define_1() {
        let input = r#"#define FOO 1"#;
        let output = r#"#define FOO 1"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn define_trailing_comment_1() {
        let input = r#"#define FOO 1 /* comment */
int foo = 1;
"#;
        let output = r#"#define FOO 1 /* comment */
int foo = 1;
"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn define_trailing_comment_3() {
        let input = r#"#define FOO bar /**< documentation */
#include "file.sp"
#if defined BAZ
int foo;

#endif
"#;
        let output = r#"#define FOO bar /**< documentation */
#include "file.sp"




"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
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

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
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

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn if_directive_expansion_3() {
        let input = r#"#define FOO(%0,%1) %0 + %1
#define BAR(%0,%1) %0 + %1
#if FOO(1, BAR(2, 3 + 4)) == 10
#endif
"#;
        let output = r#"#define FOO(%0,%1) %0 + %1
#define BAR(%0,%1) %0 + %1


"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
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

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
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

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
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

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn elseif_directive_expansion_1() {
        let input = r#"#define FOO 1
#if FOO == 2
int foo;
#elseif FOO == 1
int bar;
#endif
"#;
        let output = r#"#define FOO 1



int bar;

"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn elseif_directive_expansion_2() {
        let input = r#"#define FOO 1
#if FOO == 1
int foo;
#elseif FOO == 2
int bar;
#endif
"#;
        let output = r#"#define FOO 1

int foo;



"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn elseif_directive_expansion_3() {
        let input = r#"#define FOO 1
#if FOO == 1
    #if FOO == 1
    int foo;
    #endif
#elseif FOO == 2
int bar;
#endif
"#;
        let output = r#"#define FOO 1


    int foo;




"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn elseif_directive_expansion_4() {
        let input = r#"#define FOO 3
#if FOO == 1
int foo;
#elseif FOO == 2
int bar;
#else
int baz;
#endif
"#;
        let output = r#"#define FOO 3





int baz;

"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn elseif_directive_expansion_5() {
        let input = r#"#define FOO 3
#if FOO == 1
#if FOO == 3
int foo;
#endif
#elseif FOO == 2
int bar;
#else
int baz;
#endif
"#;
        let output = r#"#define FOO 3







int baz;

"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn define_expansion_1() {
        let input = r#"#define FOO 1
int foo = FOO;
"#;
        let output = r#"#define FOO 1
int foo = 1;
"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn define_expansion_2() {
        let input = r#"#define FOO "test"
char foo[64] = FOO;
"#;
        let output = r#"#define FOO "test"
char foo[64] = "test";
"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn define_expansion_comment_1() {
        let input = r#"#define FOO 1 //comment
int foo = FOO;
"#;
        let output = r#"#define FOO 1 //comment
int foo = 1;
"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn define_expansion_comment_2() {
        let input = r#"#define FOO 1 /* comment */
int foo = FOO;
"#;
        let output = r#"#define FOO 1 /* comment */
int foo = 1;
"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
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

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
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

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
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

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
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

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
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

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
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

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
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

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn macro_expansion_1() {
        let input = r#"#define FOO(%0,%1) %0 + %1
int foo = FOO(1, 2);
"#;
        let output = r#"#define FOO(%0,%1) %0 + %1
int foo = 1 + 2;
"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn macro_expansion_2() {
        let input = r#"#define FOO(%0) %0 %%2
int foo = FOO(2);
"#;
        let output = r#"#define FOO(%0) %0 %%2
int foo = 2 %2;
"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
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

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn macro_expansion_4() {
        let input = r#"#define FOO(%1,%2) %1 + %2
int foo = FOO(1, 2);
"#;
        let output = r#"#define FOO(%1,%2) %1 + %2
int foo = 1 + 2;
"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn macro_expansion_5() {
        let input = r#"#define FOO(%1) int %1
FOO(foo, bar);
"#;
        let output = r#"#define FOO(%1) int %1
int foo, bar;
"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn macro_expansion_6() {
        let input = r#"#define GET_VALUE(%1,%2) \
    public %1 Get%2(){ \
        %1 i; \
        this.GetValue("m_" ... #%2, i); \
        return i;}
        
        GET_VALUE(void, Foo)"#;
        let output = r#"#define GET_VALUE(%1,%2) \
    public %1 Get%2(){ \
        %1 i; \
        this.GetValue("m_" ... #%2, i); \
        return i;}
        
    public void GetFoo(){        void i;        this.GetValue("m_" ... "Foo", i);        return i;}"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn include_directive_1() {
        let input = r#"#include <sourcemod>"#;
        let output = r#"#include <sourcemod>"#;

        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn include_directive_2() {
        let input = r#"#include <sourcemod\
>"#;
        let output = r#"#include <sourcemod>
"#;
        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn include_directive_3() {
        let input = r#"#include <sourcemod>
"#;
        let output = r#"#include <sourcemod>
"#;
        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn stringizing_1() {
        let input = r#"#define FOO(%0) #%0
char foo[8] = FOO(foo);"#;
        let output = r#"#define FOO(%0) #%0
char foo[8] = "foo";"#;
        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn stringizing_2() {
        let input = r#"#define FOO(%0,%1) #%0 ... #%1
char foo[8] = FOO(foo, bar);"#;
        let output = r#"#define FOO(%0,%1) #%0 ... #%1
char foo[8] = "foo" ... "bar";"#;
        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn stringizing_3() {
        let input = r#"#define FOO(%0) #%0
char foo[8] = FOO(foo , bar);"#;
        let output = r#"#define FOO(%0) #%0
char foo[8] = "foo , bar";"#;
        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap(),
            output
        );
    }

    #[test]
    fn stringizing_10() {
        let input = r#"#define DISPOSE_MEMBER(%1) \
    Handle m_h%1; \
    if(this.GetValue("m_" ... #%1, m_h%1)){ \
        delete m_h%1;}
void foo(){
    DISPOSE_MEMBER(Foo)
}"#;
        let output = r#"#define DISPOSE_MEMBER(%1) \
    Handle m_h%1; \
    if(this.GetValue("m_" ... #%1, m_h%1)){ \
        delete m_h%1;}
void foo(){
    Handle m_hFoo;    if(this.GetValue("m_" ... "Foo", m_hFoo)){        delete m_hFoo;}
}"#;
        assert_eq!(
            SourcepawnPreprocessor::new(
                Arc::new(Url::parse("https://example.net").unwrap()),
                input
            )
            .preprocess_input(&mut extend_macros)
            .unwrap_or_else(|err| {
                eprintln!("{:?}", err);
                "".to_string()
            }),
            output
        );
    }
}
