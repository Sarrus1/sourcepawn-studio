use insta::assert_json_snapshot;
use sourcepawn_lsp::fixture::goto_definition;

#[test]
fn define_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO
         |
         ^"#,
    ));
}

#[test]
fn define_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO 1
         |
         ^"#,
    ));
}

#[test]
fn define_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO 1
int foo = FOO;
           |
           ^"#,
    ));
}

#[test]
fn define_4() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO
#if defined FOO
             |
             ^
#endif
"#,
    ));
}

#[test]
fn macro_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO(%1) %1
         |
         ^
"#,
    ));
}

#[test]
fn macro_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO(%1) %1
int foo = FOO(1);
           |
           ^
"#,
    ));
}

#[test]
fn macro_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO(%1) %1
#if defined FOO
             |
             ^
#endif
"#,
    ));
}

#[test]
fn macro_4() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO(%1) %1
int foo;
int bar = FOO(foo)
               |
               ^
"#,
    ));
}
