use insta::assert_json_snapshot;
use sourcepawn_studio::fixture::goto_definition;

#[test]
fn enum_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum Foo {
      |
      ^
}
"#,
    ));
}

#[test]
fn enum_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum Foo {
    Bar
     |
     ^
}
"#,
    ));
}

#[test]
fn enum_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum Foo {
    Bar,
    Baz
}

Foo foo() {
    return Bar;
           |
           ^
};
"#,
    ));
}

#[test]
fn anon_enum_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum {
    Bar
     |
     ^
}
"#,
    ));
}

#[test]
fn anon_enum_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum {
    Bar,
}

int foo() {
    return Bar;
           |
           ^
};
"#,
    ));
}
