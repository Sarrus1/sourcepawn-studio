use insta::assert_json_snapshot;

use sourcepawn_lsp::fixture::goto_definition;

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
