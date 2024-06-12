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
#include "bar.sp"

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

#[test]
fn include_1() {
    assert_json_snapshot!(complete(
        r#"
%! bar.sp
int bar;
%! include/baz.inc
int baz;
%! foo.sp
#include ""
          |
          ^"#,
        Some('"'.to_string())
    ));
}

#[test]
fn include_2() {
    assert_json_snapshot!(complete(
        r#"
%! bar.sp
int bar;
%! include/baz.inc
int baz;
%! foo.sp
#include <>
          |
          ^"#,
        Some('<'.to_string())
    ));
}

#[test]
fn include_3() {
    assert_json_snapshot!(complete(
        r#"
%! bar.sp
int bar;
%! include/baz.inc
#include <>
          |
          ^
%! foo.sp
int foo;"#,
        Some('<'.to_string())
    ));
}

#[test]
fn include_4() {
    assert_json_snapshot!(complete(
        r#"
%! bar.sp
int bar;
%! include/baz.inc
#include ""
          |
          ^
%! foo.sp
int foo;"#,
        Some('"'.to_string())
    ));
}

#[test]
fn include_5() {
    assert_json_snapshot!(complete(
        r#"
%! bar.sp
int bar;
%! include/baz.inc
#include ""
          |
          ^
%! include/foo.inc
int foo;"#,
        Some('"'.to_string())
    ));
}

#[test]
fn include_6() {
    assert_json_snapshot!(complete(
        r#"
%! main.sp
#include "sub_folder/foo.sp"
int main;
%! sub_folder/foo.sp
#include <sub_folder/>
                     |
                     ^
%! sub_folder/bar.sp
int bar;"#,
        Some("/".to_string())
    ));
}
