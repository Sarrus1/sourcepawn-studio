use insta::assert_json_snapshot;
use sourcepawn_lsp::fixture::hover;

#[test]
fn function_1() {
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
