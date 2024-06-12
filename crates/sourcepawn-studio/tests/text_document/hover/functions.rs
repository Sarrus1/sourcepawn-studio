use insta::assert_json_snapshot;
use sourcepawn_studio::fixture::hover;

#[test]
fn function_1() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
/**
 * The foo function.
 */
void foo(){}
      |
      ^
"#,
    ));
}

#[test]
fn function_2() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
/**
 * The foo function.
 */
#pragma deprecated Use bar instead.
void foo(){}
      |
      ^
"#,
    ));
}

#[test]
fn function_3() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
/**
 * The foo function.
 * @param bar The bar parameter.
 *            It is useful.
 * @param baz The baz parameter.
 */
void foo(int bar, int baz){}
              |
              ^
"#,
    ));
}

#[test]
fn function_4() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
/**
 * The foo function.
 * @param bar The bar parameter.
 *            It is useful.
 * @param baz The baz parameter.
 */
void foo(int bar,
      |
      ^
         int baz){}
"#,
    ));
}
