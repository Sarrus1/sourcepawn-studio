use insta::assert_json_snapshot;
use sourcepawn_studio::fixture::complete;

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
#include "sub_folder/"
                     |
                     ^
int main;
%! sub_folder/foo.sp
int foo;"#,
        Some("/".to_string())
    ));
}
