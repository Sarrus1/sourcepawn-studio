use insta::assert_json_snapshot;
use sourcepawn_studio::fixture::goto_definition;

#[test]
fn old_global_variable_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
new foo;
     |
     ^             
"#,
    ));
}

#[test]
fn old_global_variable_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
new _:foo = 0, Float:bar, bool:baz = true, String:qux = 'c';
                                                   |
                                                   ^             
"#,
    ));
}

#[test]
fn old_global_variable_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
new foo;
bar() {
    foo = 1;
     |
     ^
}
"#,
    ));
}
