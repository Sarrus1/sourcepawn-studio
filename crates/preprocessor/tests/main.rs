use vfs::FileId;

use insta::assert_snapshot;

fn extend_macros(
    _macro_store: &mut MacrosMap,
    mut _path: String,
    _file_id: FileId,
    _quoted: bool,
) -> anyhow::Result<()> {
    Ok(())
}

#[allow(unused_macros)]
macro_rules! assert_preproc_eq {
    ($input:expr) => {
        assert_snapshot!(
            SourcepawnPreprocessor::new(FileId::from(0), $input, &mut extend_macros)
                .preprocess_input()
                .preprocessed_text()
                .as_ref()
        );
    };
}

use preprocessor::{MacrosMap, SourcepawnPreprocessor};
#[test]
fn no_preprocessor_directives() {
    let input = r#"
        int foo;
        int bar;
        "#;

    assert_preproc_eq!(input);
}

#[test]
fn if_directive_defined() {
    let input = r#"#define FOO
#if defined FOO
    int foo;
#endif"#;

    assert_preproc_eq!(input);
}

#[test]
fn if_directive_defined_complex_1() {
    let input = r#"#define FOO
#if defined FOO && defined BAR
    int foo;
    int bar;
#endif"#;

    assert_preproc_eq!(input);
}

#[test]
fn if_directive_defined_complex_2() {
    let input = r#"#define FOO
#define BAR
#if defined FOO && defined BAR
    int foo;
    int bar;
#endif"#;

    assert_preproc_eq!(input);
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

    assert_preproc_eq!(input);
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

    assert_preproc_eq!(input);
}

#[test]
fn if_directive_defined_complex_5() {
    let input = r#"#define FOO
#if defined FOO
int foo;
#else
int bar;
#endif"#;

    assert_preproc_eq!(input);
}

#[test]
fn if_directive_defined_complex_6() {
    let input = r#"#define FOO
#if defined BAR
int foo;
#else
int bar;
#endif"#;

    assert_preproc_eq!(input);
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

    assert_preproc_eq!(input);
}

#[test]
fn if_directive_defined_complex_8() {
    let input = r#"#define FOO 1
#if defined FOO
int foo;
#elseif FOO == 1
int bar;
#endif"#;

    assert_preproc_eq!(input);
}

#[test]
fn define_1() {
    let input = r#"#define FOO 1"#;

    assert_preproc_eq!(input);
}

#[test]
fn define_trailing_comment_1() {
    let input = r#"#define FOO 1 /* comment */
int foo = 1;
"#;

    assert_preproc_eq!(input);
}

#[test]
fn define_trailing_comment_3() {
    let input = r#"#define FOO bar /**< documentation */
#include "file.sp"
#if defined BAZ
int foo;

#endif
"#;

    assert_preproc_eq!(input);
}

#[test]
fn if_directive_expansion_1() {
    let input = r#"#define FOO 1
#if FOO
    int foo;
#endif"#;

    assert_preproc_eq!(input);
}

#[test]
fn if_directive_expansion_2() {
    let input = r#"#define FOO 1
#if FOO == 1
    int foo;
#endif"#;

    assert_preproc_eq!(input);
}

#[test]
fn if_directive_expansion_3() {
    let input = r#"#define FOO(%0,%1) %0 + %1
#define BAR(%0,%1) %0 + %1
#if FOO(1, BAR(2, 3 + 4)) == 10
#endif
"#;

    assert_preproc_eq!(input);
}

#[test]
fn if_directive_nested_expansion_1() {
    let input = r#"#define FOO BAR
#define BAR 1
#if FOO == 1
    int foo;
#endif"#;

    assert_preproc_eq!(input);
}

#[test]
fn if_directive_nested_expansion_2() {
    let input = r#"#define FOO BAR + 4
#define BAR 1 + BAZ
#define BAZ 2 + 3
#if FOO == 10
    int foo;
#endif"#;

    assert_preproc_eq!(input);
}

#[test]
fn if_directive_nested_expansion_infinite_loop_1() {
    let input = r#"#define FOO BAR
#define BAR FOO
#if FOO == 1
    int foo;
#endif"#;

    assert_preproc_eq!(input);
}

#[test]
fn multiline_block_comment_1() {
    let input = r#"int foo;
#if false
  /*
    A
block
            comment
        */
#endif
int bar;
"#;

    assert_preproc_eq!(input);
}

#[test]
fn multiline_block_comment_2() {
    let input = r#"int foo;
#if false /*
    A
block
            comment
        */ #endif
int bar;
"#;

    assert_preproc_eq!(input);
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

    assert_preproc_eq!(input);
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

    assert_preproc_eq!(input);
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

    assert_preproc_eq!(input);
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

    assert_preproc_eq!(input);
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

    assert_preproc_eq!(input);
}

#[test]
fn define_undef_1() {
    let input = r#"#define FOO 1
#undef FOO
"#;

    assert_preproc_eq!(input);
}

#[test]
fn define_expansion_1() {
    let input = r#"#define FOO 1
int foo = FOO;
"#;

    assert_preproc_eq!(input);
}

#[test]
fn define_expansion_2() {
    let input = r#"#define FOO "test"
char foo[64] = FOO;
"#;

    assert_preproc_eq!(input);
}

#[test]
fn define_expansion_comment_1() {
    let input = r#"#define FOO 1 //comment
int foo = FOO;
"#;

    assert_preproc_eq!(input);
}

#[test]
fn define_expansion_comment_2() {
    let input = r#"#define FOO 1 /* comment */
int foo = FOO;
"#;

    assert_preproc_eq!(input);
}

#[test]
fn define_expansion_comment_3() {
    let input = r#"#define FOO 1 /* long\
comment */
int foo = FOO;
"#;

    assert_preproc_eq!(input);
}

#[test]
fn define_expansion_comment_4() {
    let input = r#"#define FOO 1 /* long\
comment */ + 2
int foo = FOO;
int bar;
"#;

    assert_preproc_eq!(input);
}

#[test]
fn define_expansion_comment_5() {
    let input = r#"#define FOO 1 /* long\
comment */ + 2 // Line comment
int foo = FOO;
int bar;
"#;

    assert_preproc_eq!(input);
}

#[test]
fn define_expansion_line_continuation_1() {
    let input = r#"#define FOO "test \
expansion"
char foo[64] = FOO;
"#;

    assert_preproc_eq!(input);
}

#[test]
fn define_expansion_line_continuation_2() {
    let input = r#"#define FOO "test \
expansion \
\
also"
char foo[64] = FOO;
"#;

    assert_preproc_eq!(input);
}

#[test]
fn define_expansion_nested_1() {
    let input = r#"#define FOO BAR +   2
#define BAR 1
int foo = FOO;
"#;

    assert_preproc_eq!(input);
}

#[test]
fn define_expansion_nested_2() {
    let input = r#"#define FOO BAR + 3
#define BAR 1 + BAZ
#define BAZ 2
int foo = FOO;
"#;

    assert_preproc_eq!(input);
}

#[test]
fn define_expansion_nested_3() {
    let input = r#"#define FOO BAR + 3
#define BAR 1 + 2
int foo = FOO;
"#;

    assert_preproc_eq!(input);
}

#[test]
fn define_expansion_nested_4() {
    let input = r#"#define FOO 1 + 2
#define BAR FOO + 3 + 4
int bar = BAR;
"#;

    assert_preproc_eq!(input);
}

#[test]
fn macro_expansion_1() {
    let input = r#"#define FOO(%0,%1) %0 + %1
int foo = FOO(1, 2);
"#;

    assert_preproc_eq!(input);
}

#[test]
fn macro_expansion_2() {
    let input = r#"#define FOO(%0) %0 %%2
int foo = FOO(2);
"#;

    assert_preproc_eq!(input);
}

#[test]
fn macro_expansion_3() {
    let input = r#"#define FOO(%0,%1) %0 + %1
#define BAR(%0,%1) %0 + %1
int foo = FOO(1, BAR(2, 3 + 4));
"#;

    assert_preproc_eq!(input);
}

#[test]
fn macro_expansion_4() {
    let input = r#"#define FOO(%1,%2) %1 + %2
int foo = FOO(1, 2);
"#;

    assert_preproc_eq!(input);
}

#[test]
fn macro_expansion_5() {
    let input = r#"#define FOO(%1) int %1
FOO(foo, bar);
"#;

    assert_preproc_eq!(input);
}

#[test]
fn macro_expansion_6() {
    let input = r#"#define GET_VALUE(%1,%2) \
    public %1 Get%2(){ \
        %1 i; \
        this.GetValue("m_" ... #%2, i); \
        return i;}
        
        GET_VALUE(void, Foo)"#;

    assert_preproc_eq!(input);
}

#[test]
fn macro_expansion_7() {
    let input = r#"#define FOO(%0,%1) %0 + %1
#define BAR(%0,%1) 1 + FOO(%0, %1)
int foo = BAR(2, 3)
"#;

    assert_preproc_eq!(input);
}

#[test]
fn macro_no_expansion_1() {
    let input = r#"#define FOO(%1) #%1
public void OnPluginStart() {
    PrintToServer(FOO /*foo*/ (foo));
}
"#;

    assert_preproc_eq!(input);
}

#[test]
fn macro_no_expansion_2() {
    let input = r#"#define FOO(%1) #%1
public void OnPluginStart() {
    PrintToServer(FOO
        (foo));
}
"#;
    assert_preproc_eq!(input);
}

#[test]
fn macro_no_expansion_3() {
    let input = r#"#define FOO(%1) #%1
public void OnPluginStart() {
    PrintToServer(FOO);
}
"#;
    assert_preproc_eq!(input);
}

#[test]
fn macro_no_expansion_4() {
    let input = r#"#define FOO(%1) %1
#define BAR(%1) %1
public void OnPluginStart() {
    PrintToServer(BAR(FOO));
}
"#;

    assert_preproc_eq!(input);
}

#[test]
fn include_directive_1() {
    let input = r#"#include <sourcemod>"#;

    assert_preproc_eq!(input);
}

#[test]
fn include_directive_2() {
    let input = r#"#include <sourcemod\
>"#;

    assert_preproc_eq!(input);
}

#[test]
fn include_directive_3() {
    let input = r#"#include <sourcemod>
"#;

    assert_preproc_eq!(input);
}

#[test]
fn stringizing_1() {
    let input = r#"#define FOO(%0) #%0
char foo[8] = FOO(foo);"#;

    assert_preproc_eq!(input);
}

#[test]
fn stringizing_2() {
    let input = r#"#define FOO(%0,%1) #%0 ... #%1
char foo[8] = FOO(foo, bar);"#;

    assert_preproc_eq!(input);
}

#[test]
fn stringizing_3() {
    let input = r#"#define FOO(%0) #%0
char foo[8] = FOO(foo , bar);"#;

    assert_preproc_eq!(input);
}

#[test]
fn stringizing_4() {
    let input = r#"#define DISPOSE_MEMBER(%1) \
    Handle m_h%1; \
    if(this.GetValue("m_" ... #%1, m_h%1)){ \
        delete m_h%1;}
void foo(){
    DISPOSE_MEMBER(Foo)
}"#;

    assert_preproc_eq!(input);
}
