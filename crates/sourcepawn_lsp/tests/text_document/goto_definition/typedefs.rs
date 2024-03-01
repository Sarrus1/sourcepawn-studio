use insta::assert_json_snapshot;
use sourcepawn_lsp::fixture::goto_definition;

#[test]
fn typedef_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
typedef Foo = function int ();
         |
         ^
"#,
    ));
}

#[test]
fn typedef_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
typedef Foo = function int (int foo);
                                 |
                                 ^
"#,
    ));
}

#[test]
fn typedef_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum Bar {}
typedef Foo = function Bar ();
                        |
                        ^
"#,
    ));
}

#[test]
fn typedef_4() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum Bar {}
typedef Foo = function Bar (Bar bar);
                             |
                             ^
"#,
    ));
}

#[test]
fn typeset_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
typeset Foo {
         |
         ^
    function Bar ();
}
"#,
    ));
}

#[test]
fn typeset_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
typeset Foo {
    function (int foo);
                   |
                   ^
}
"#,
    ));
}

#[test]
fn functag_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
functag public int:Foo();
                    |
                    ^
"#,
    ));
}

#[test]
fn functag_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
functag public int:Foo(args);
                        |
                        ^
"#,
    ));
}

#[test]
fn functag_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum Bar {}
functag public int:Foo(Bar:args);
                        |
                        ^
"#,
    ));
}

#[test]
fn functag_4() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum Bar {}
functag public Bar:Foo();
                |
                ^
"#,
    ));
}
