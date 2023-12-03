use insta::assert_json_snapshot;

use sourcepawn_lsp::fixture::goto_definition;

#[test]
fn global_variable_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
int foo;
     |
     ^"#,
    ));
}

#[test]
fn global_variable_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
int foo = 1;
     |
     ^"#,
    ));
}

#[test]
fn global_variable_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
int foo, bar = 1;
          |
          ^"#,
    ));
}

#[test]
fn local_variable_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
int foo;

void bar() {
    foo = 1;
    |
    ^
}
"#,
    ));
}
