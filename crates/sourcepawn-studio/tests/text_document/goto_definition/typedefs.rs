use insta::assert_json_snapshot;
use sourcepawn_studio::fixture::goto_definition;

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
    function (int foo, int bar);
}
"#,
    ));
}

#[test]
fn typeset_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum Bar {}
typeset Foo {
    function (Bar foo);
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

#[test]
fn funcenum_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
funcenum Foo {
          |
          ^
    int:public(),
}
"#,
    ));
}

#[test]
fn funcenum_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
funcenum Foo {
    int:public(foo),
                |
                ^
    int:public(foo, bar),

}
"#,
    ));
}

#[test]
fn funcenum_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum Bar {}
funcenum Foo {
    int:public(Bar:foo),
                |
                ^
}
"#,
    ));
}
