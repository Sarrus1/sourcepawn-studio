use insta::assert_json_snapshot;
use sourcepawn_lsp::fixture::hover;

#[test]
fn macro_1() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
#define FOO
FOO
 |
 ^
"#,
    ));
}

#[test]
fn macro_2() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
#define FOO 1
int foo = FOO;
           |
           ^
"#,
    ));
}

#[test]
fn macro_3() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
#define FOO(%1) %1 + %1
int foo = 1;
int bar = FOO(foo);
           |
           ^
"#,
    ));
}
