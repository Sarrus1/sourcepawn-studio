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
int foo;
int bar = FOO(foo);
           |
           ^
"#,
    ));
}

#[test]
fn macro_4() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
#define FOO 1 + 1
int foo = FOO + FOO;
                 |
                 ^
"#,
    ));
}

#[test]
fn macro_5() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
#define FOO(%1) %1 + %1
int foo;
int bar = FOO(foo) + FOO(foo);
                      |
                      ^
"#,
    ));
}

#[test]
fn macro_6() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
#define FOO(%0) 	view_as<int>( %0 )
#define BAR(%0,%1) foo[FOO( %0 )][%1]
#define BAZ      (1 << 0)
int foo[10][10];
int bar = BAR( 1, 2 ) + BAZ;
                         |
                         ^
"#,
    ));
}

#[test]
fn macro_7() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
#define FOO(%0) 	view_as<int>( %0 )
#define BAR(%0,%1) foo[FOO( %0 )][%1]
#define BAZ      (1 << 0)
int foo[10][10];
int bar = BAR( FOO(1), 2 ) + BAZ;
                              |
                              ^
"#,
    ));
}

#[test]
fn macro_8() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
enum Bar {Bar1}
#define FOO(%0,%1) view_as<%0>( %1 )
#define BAR(%0)                     FOO( Bar, %0 )
Bar bar = BAR( 1 );
           |
           ^
"#,
    ));
}

#[test]
fn macro_9() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
enum Bar {Bar1}
#define FOO view_as<Bar>( 1 )
#define BAR                     FOO
Bar bar = BAR;
           |
           ^
"#,
    ));
}

#[test]
fn macro_10() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
#define FOO  "foo"
char foo[10] = FOO;
                |
                ^
"#,
    ));
}

#[test]
fn macro_11() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
#define MAXPLAYERS      101  /**< Maximum number of players SourceMod supports */
int foo = MAXPLAYERS;
              |
              ^
"#,
    ));
}

#[test]
fn macro_12() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
#pragma deprecated Use something else
#define MAXPLAYERS      101  /**< Maximum number of players SourceMod supports */
int foo = MAXPLAYERS;
              |
              ^
"#,
    ));
}
