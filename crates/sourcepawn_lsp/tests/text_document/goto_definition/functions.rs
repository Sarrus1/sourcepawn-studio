use insta::assert_json_snapshot;
use sourcepawn_lsp::fixture::goto_definition;

#[test]
fn function_named_arg_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
void bar(int foo=1) {}

void baz() {
    bar(.foo=1);
          |
          ^
}
"#,
    ));
}

#[test]
fn function_named_arg_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
int bar(int foo=1) {}
int baz(int foo=1) {}

void foo() {
    bar(.foo=baz(.foo=1));
                   |
                   ^
}
"#,
    ));
}

#[test]
fn method_named_arg_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum struct Foo {
    void foo(int foo=1) {}
}

void bar() {
    Foo foo;
    foo.foo(.foo=1);
              |
              ^
}
"#,
    ));
}
