use insta::assert_json_snapshot;
use sourcepawn_studio::fixture::goto_definition;

#[test]
fn forward_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
forward int Foo(int bar);
             |
             ^
"#,
    ));
}

#[test]
fn forward_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
forward int Foo(int bar);
                     |
                     ^
"#,
    ));
}

#[test]
fn native_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
native int Foo(int bar);
             |
             ^
"#,
    ));
}

#[test]
fn native_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
native int Foo(int bar);
                     |
                     ^
"#,
    ));
}
