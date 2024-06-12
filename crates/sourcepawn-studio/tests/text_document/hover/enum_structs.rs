use insta::assert_json_snapshot;
use sourcepawn_studio::fixture::hover;

#[test]
fn enum_struct_1() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
/**
 * The Foo enum struct.
 */
enum struct Foo {}
             |
             ^
"#,
    ));
}

#[test]
fn enum_struct_2() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
/**
 * The Foo enum struct.
 */
#pragma deprecated Use Bar instead.
enum struct Foo {}
             |
             ^
"#,
    ));
}

#[test]
fn enum_struct_3() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
enum struct Foo {
    /**
     * The foo field.
     */
    int foo;
         |
         ^
}
"#,
    ));
}

#[test]
fn enum_struct_4() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
enum struct Foo {
    /**
     * The foo field.
     */
    #pragma deprecated Use bar instead.
    int foo;
         |
         ^
}
"#,
    ));
}

#[test]
fn enum_struct_5() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
enum struct Foo {
    /**
     * The foo method.
     */
    Foo foo() {}
         |
         ^
}
"#,
    ));
}

#[test]
fn enum_struct_6() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
enum struct Foo {
    /**
     * The foo method.
     */
    #pragma deprecated Use bar instead.
    Foo foo() {}
         |
         ^
}
"#,
    ));
}
