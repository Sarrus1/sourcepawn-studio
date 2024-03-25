use insta::assert_json_snapshot;
use sourcepawn_lsp::fixture::goto_definition;

#[test]
fn define_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO
         |
         ^"#,
    ));
}

#[test]
fn define_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO 1
         |
         ^"#,
    ));
}

#[test]
fn define_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO 1
int foo = FOO;
           |
           ^"#,
    ));
}

#[test]
fn define_4() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO
#if defined FOO
             |
             ^
#endif
"#,
    ));
}

#[test]
fn macro_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO(%1) %1
         |
         ^
"#,
    ));
}

#[test]
fn macro_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO(%1) %1
int foo = FOO(1);
           |
           ^
"#,
    ));
}

#[test]
fn macro_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO(%1) %1
#if defined FOO
             |
             ^
#endif
"#,
    ));
}

#[test]
fn macro_4() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO(%1) %1
int foo;
int bar = FOO(foo);
               |
               ^
"#,
    ));
}

#[test]
fn macro_5() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO(%1)    %1
int foo;
int bar = FOO( foo )
                |
                ^
"#,
    ));
}

#[test]
fn macro_6() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO(%1,%2)    %1 +     %2
int foo;
int bar = FOO( foo    , foo )
                         |
                         ^
"#,
    ));
}

#[test]
fn macro_7() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO(%1,%2)    %1 +     %2
#define BAAAAAR bar
int foo;
int bar;
int baz = BAAAAAR + FOO( foo    , bar );
                                   |
                                   ^
"#,
    ));
}

#[test]
fn macro_8() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO(%1,%2)    %1 +     %2
int foo;
int bar;
int baz = FOO( foo    , foo ) + FOO( foo    , bar );
                                               |
                                               ^
"#,
    ));
}

#[test]
fn macro_9() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum Foo {
    Foo1,
}
#define BAR(%0,%1) view_as<%0>( %1 )
#define FOO(%0)    BAR( Foo, %0 )
#define BAZ(%0)    sizeof   %0

void bar(int foo, Foo foo2, int foo3, bool foo4  = true) {}

void baz() {
    int foo5 = 0;
    char foo6[10];
    bar( foo5, FOO( foo5 ), FOO( foo6 ), .foo4 = foo5 );
                      |
                      ^
}
"#,
    ));
}

#[test]
fn macro_10() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum Foo {
    Foo1,
}
#define BAR(%0,%1) view_as<%0>( %1 )
#define FOO(%0)    BAR( Foo, %0 )
#define BAZ(%0)    sizeof   %0

void bar(int foo, Foo foo2, int foo3, bool foo4  = true) {}

void baz() {
    int foo5 = 0;
    char foo6[10];
    bar( foo5, FOO( foo5 ), FOO( foo6 ), .foo4 = foo5 );
                                           |
                                           ^
}
"#,
    ));
}

#[test]
fn preprocessor_offsetting_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO foo
int foo;
int bar = FOO + foo;
                 |
                 ^
"#,
    ));
}

#[test]
fn preprocessor_offsetting_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO foo + foo
int foo;
int baz;
int bar = FOO + baz;
                 |
                 ^
"#,
    ));
}

#[test]
fn preprocessor_offsetting_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO foo + foo
int foo;
int baz;
int bar = FOO + FOO + baz;
                       |
                       ^
"#,
    ));
}

#[test]
fn preprocessor_offsetting_4() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOOOOOOOOOOOOOOO foo
int foo;
int baz;
int bar = FOOOOOOOOOOOOOOO + baz;
                              |
                              ^
"#,
    ));
}

#[test]
fn preprocessor_offsetting_5() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOOOOOOOOOOOOOOO foo
int foo;
int baz;
int bar = FOOOOOOOOOOOOOOO + FOOOOOOOOOOOOOOO + baz;
                                                 |
                                                 ^
"#,
    ));
}

#[test]
fn preprocessor_offsetting_6() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOOOOOOOOOOOOOOO int foo;
FOOOOOOOOOOOOOOO int bar;
int baz = bar;
           |
           ^
"#,
    ));
}

#[test]
fn preprocessor_offsetting_7() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOOOOOOOOOOOOOOO int foo;
#define BAAAAAAAAAAAAAAR int bar;
FOOOOOOOOOOOOOOO BAAAAAAAAAAAAAAR int baz;
int qux = baz;
           |
           ^
"#,
    ));
}

#[test]
fn preprocessor_disable_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO
#if defined FOO
int foo;
#endif
void bar() {
    foo = 1;
     |
     ^
}
"#,
    ));
}

#[test]
fn preprocessor_disable_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define FOO
#include "foo.sp"

%! foo.sp
#if defined FOO
int foo;
#endif
void bar() {
    foo = 1;
     |
     ^
}
"#,
    ));
}

#[test]
fn preprocessor_disable_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#include "foo.sp"
#if defined FOO
int foo;
#endif
void bar() {
    foo = 1;
     |
     ^
}

%! foo.sp
#define FOO
"#,
    ));
}
