use insta::assert_json_snapshot;
use sourcepawn_studio::fixture::signature_help;

#[test]
fn function_1() {
    assert_json_snapshot!(signature_help(
        r#"
%! main.sp
void foo() {
    foo();
        |
        ^
}
"#,
    ));
}

#[test]
fn function_2() {
    assert_json_snapshot!(signature_help(
        r#"
%! main.sp
void foo(int bar) {
    foo();
        |
        ^
}
"#,
    ));
}

#[test]
fn function_3() {
    assert_json_snapshot!(signature_help(
        r#"
%! main.sp
void foo(int bar) {
    foo(bar);
           |
           ^
}
"#,
    ));
}

#[test]
fn function_4() {
    assert_json_snapshot!(signature_help(
        r#"
%! main.sp
void foo(int bar, int baz) {
    foo(bar,);
            |
            ^
}
"#,
    ));
}

#[test]
fn function_5() {
    assert_json_snapshot!(signature_help(
        r#"
%! main.sp
/**
 * @param bar    This is the bar parameter
 * @param baz    This is the baz parameter, it's a long description
 */
void foo(int bar, int baz) {
    foo(bar,);
            |
            ^
}
"#,
    ));
}

#[test]
fn function_6() {
    assert_json_snapshot!(signature_help(
        r#"
%! main.sp
#include "foo.sp"
void qux(int bar, int baz) {
    foo(bar,);
            |
            ^
}

%! foo.sp
/**
 * @param bar    This is the bar parameter
 * @param baz    This is the baz parameter, it's a long description
 */
void foo(int bar, int baz) {}
"#,
    ));
}

#[test]
fn function_7() {
    assert_json_snapshot!(signature_help(
        r#"
%! main.sp
/**
 * @param bar    This is the bar parameter
 * @param ...    This is the rest parameter
 */
void foo(int bar, any ...) {
    foo(bar, 0, 1);
                 |
                 ^
}
"#,
    ));
}
