use insta::assert_json_snapshot;
use sourcepawn_studio::fixture::goto_definition;

#[test]
fn enum_struct_scope_access_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum struct Foo {
    int foo[8];
}

void main() {
    Foo::foo;
          |
          ^
}
"#,
    ));
}
