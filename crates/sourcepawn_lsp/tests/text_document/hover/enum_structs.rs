use insta::assert_json_snapshot;
use sourcepawn_lsp::fixture::hover;

#[test]
fn macro_13() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
enum struct Foo {
    void bar() {}
          |
          ^
}
"#,
    ));
}
