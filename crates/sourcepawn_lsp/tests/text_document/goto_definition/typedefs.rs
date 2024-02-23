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
