use insta::assert_json_snapshot;
use sourcepawn_lsp::fixture::hover;

#[test]
fn global_1() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
public const int MaxClients;   /**< Maximum number of players the server supports (dynamic) */
                    |
                    ^
"#,
    ));
}
