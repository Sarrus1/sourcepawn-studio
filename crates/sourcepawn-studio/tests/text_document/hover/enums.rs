use insta::assert_json_snapshot;
use sourcepawn_studio::fixture::hover;

#[test]
fn enum_1() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
/**
 * The Foo enum.
 */
enum Foo {}
      |
      ^
"#,
    ));
}

#[test]
fn enum_2() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
/**
 * The Foo enum.
 */
#pragma deprecated Use Bar instead.
enum Foo {}
      |
      ^
"#,
    ));
}

#[test]
fn enum_3() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
enum Foo {
    Foo1, /** The Foo1 variant. */
      |
      ^
}
"#,
    ));
}

#[test]
fn enum_4() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
enum Foo {
    #pragma deprecated Use Foo2 instead.
    Foo1, /** The Foo1 variant. */
      |
      ^
}
"#,
    ));
}
