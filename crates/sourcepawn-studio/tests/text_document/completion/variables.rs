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
fn global_variable_2() {
    assert_json_snapshot!(complete(
        r#"
%! main.sp
int foo = 1;

|
^"#,
        None
    ));
}

#[test]
fn global_variable_3() {
    assert_json_snapshot!(complete(
        r#"
%! main.sp
int foo[16] = {1, ...};

|
^"#,
        None
    ));
}

#[test]
fn global_variable_in_function() {
    assert_json_snapshot!(complete(
        r#"
%! main.sp
int foo;

void main() {

|
^
}"#,
        None
    ));
}

#[test]
fn global_variable_in_function_block() {
    assert_json_snapshot!(complete(
        r#"
%! main.sp
int foo;

void main() {
  for(;;) {
    
    |
    ^
  }
}"#,
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

#[test]
fn local_variable_in_function() {
    assert_json_snapshot!(complete(
        r#"
%! main.sp
void main() {
  int foo;
  
  |
  ^
}"#,
        None
    ));
}

#[test]
fn local_variable_outside_function() {
    assert_json_snapshot!(complete(
        r#"
%! main.sp
void main() {
  int foo;
}

|
^
"#,
        None
    ));
}

#[test]
fn local_variable_in_other_function() {
    assert_json_snapshot!(complete(
        r#"
%! main.sp
void main() {
  int foo;
}

void bar() {

|
^
}"#,
        None
    ));
}

#[test]
fn function_parameter() {
    assert_json_snapshot!(complete(
        r#"
%! main.sp
void main(int foo) {

|
^
}"#,
        None
    ));
}

#[test]
fn local_variable_for_loop() {
    assert_json_snapshot!(complete(
        r#"
%! main.sp
void main() {
  for(int foo;;) {
    
    |
    ^
  }
}"#,
        None
    ));
}
