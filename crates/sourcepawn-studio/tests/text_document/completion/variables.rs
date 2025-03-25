use insta::assert_json_snapshot;

use sourcepawn_studio::fixture::complete;

#[test]
fn global_variable_1() {
    assert_json_snapshot!(complete(
        r#"
%! main.sp
int foo;

|
^"#,
        None
    ));
}

#[test]
fn global_variable_include_1() {
    assert_json_snapshot!(complete(
        r#"
%! include/bar.sp
int foo;
%! main.sp
#include "include/bar.sp"

|
^"#,
        None
    ));
}

#[test]
fn global_variable_circular_include_1() {
    assert_json_snapshot!(complete(
        r#"
%! foo.sp
#include "bar.sp"
int foo;
%! bar.sp
#include "foo.sp"
int bar;

|
^"#,
        None
    ));
}
