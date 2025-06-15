use insta::assert_json_snapshot;
use sourcepawn_studio::fixture::goto_definition;

mod arrays;
mod enum_structs;
mod enums;
mod function_declarations;
mod functions;
mod macros;
mod methodmaps;
mod statements;
mod structs;
mod typedefs;
mod variables;

#[test]
fn global_variable_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
int foo;
     |
     ^"#,
    ));
}

#[test]
fn global_variable_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
int foo = 1;
     |
     ^"#,
    ));
}

#[test]
fn global_variable_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
int foo, bar = 1;
          |
          ^"#,
    ));
}

#[test]
fn global_variable_4() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
int foo = 1;
int bar = foo;
           |
           ^"#,
    ));
}

#[test]
fn global_variable_5() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
int foo;

void bar() {
    foo = 1;
     |
     ^
}
"#,
    ));
}

#[test]
fn global_variable_6() {
    assert_json_snapshot!(goto_definition(
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
fn global_variable_7() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#define A_REALLY_LONG_INCLUDE(%1,%2) %1 + %2
int foo;
int bar = A_REALLY_LONG_INCLUDE(foo + foo, foo);
                                 |
                                 ^             
"#,
    ));
}

#[test]
fn local_variable_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
void foo() {
    int bar = 1;
         |
         ^
}
"#,
    ));
}

#[test]
fn local_variable_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
void bar() {
    int foo;
    foo = 1;
     |
     ^
}
"#,
    ));
}

#[test]
fn local_variable_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
int foo;
void bar() {
    int foo;
    {
        int foo;
        foo = 1;
         |
         ^
    }
}
"#,
    ));
}

#[test]
fn local_variable_4() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum struct Foo {
    int foo;
}

void bar() {
    Foo foo;
    foo.foo = 1;
      |
      ^
}
"#,
    ));
}

#[test]
fn local_variable_5() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum struct Foo {
    int foo;
}

void bar() {
    Foo foo;
    int bar = foo.foo;
                   |
                   ^
}
"#,
    ));
}

#[test]
fn function_parameter_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
void bar(int foo) {
              |
              ^
}
"#,
    ));
}

#[test]
fn function_parameter_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
void bar(int foo) {
    foo = 1;
     |
     ^
}
"#,
    ));
}

#[test]
fn function_parameter_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
int foo;

void bar(int foo) {
              |
              ^
}
"#,
    ));
}

#[test]
fn function_parameter_4() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#include "foo.sp"
void bar(int foo) {
              |
              ^
}

%! foo.sp
int foo;
"#,
    ));
}

#[test]
fn function_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
void bar() {}
     |
     ^
"#,
    ));
}

#[test]
fn function_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
void bar() {
    bar();
     |
     ^
}
"#,
    ));
}

#[ignore = "We should include sourcemod for this to work"]
#[test]
fn function_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
void bar() {
    float();
      |
      ^
}
"#,
    ));
}

#[test]
fn enum_struct_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum struct Foo {
             |
             ^
    int foo;
}
"#,
    ));
}

#[test]
fn enum_struct_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum struct Foo {
    int foo;
         |
         ^
}
"#,
    ));
}

#[test]
fn enum_struct_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum struct Foo {
    int foo;
}

Foo foo;
     |
     ^

"#,
    ));
}

#[test]
fn enum_struct_4() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum struct Bar {
    int bar;
}

enum struct Foo {
    Bar bar;
     |
     ^
}
"#,
    ));
}

#[test]
fn field_access_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum struct Foo {
    int foo;
}

Foo foo;

void bar() {
    foo.foo = 1;
         |
         ^
}
"#,
    ));
}

#[test]
fn field_access_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum struct Foo {
    int foo;
}

void bar() {
    Foo foo;
    foo.foo = 1;
         |
         ^
}
"#,
    ));
}

#[test]
fn field_access_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum struct Foo {
    int foo;
}

void bar() {
    Foo foo;
    baz(foo.foo);
             |
             ^
}

void baz(int foo) {}
"#,
    ));
}

#[test]
fn method_call_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum struct Foo {
    void foo() {};
}

Foo foo;

void bar() {
    foo.foo();
         |
         ^
}
"#,
    ));
}

#[test]
fn enum_struct_method_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum struct Foo {
    void foo() {};
          |
          ^
}
"#,
    ));
}

#[test]
fn enum_struct_method_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum struct Foo {
    void foo() {
        int bar;
             |
             ^
    };
}
"#,
    ));
}

#[test]
fn enum_struct_method_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum struct Foo {
    void foo(int bar) {
                  |
                  ^
    };
}
"#,
    ));
}

#[test]
fn enum_struct_method_4() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum struct Foo {
    void foo(int bar) {
        bar = 1;
         |
         ^
    };
}
"#,
    ));
}

#[test]
fn include_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#include "foo.sp"
           |
           ^

%! foo.sp
int foo;
int bar;
"#,
    ));
}

#[test]
fn include_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#include "foo.sp"
           |
           ^

%! include/foo.sp
int foo;
"#,
    ));
}
