use insta::assert_json_snapshot;
use sourcepawn_lsp::fixture::goto_definition;

#[test]
fn array_declaration_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
void foo() {
    const bar = 1;
    int baz[10] = { bar, ... };
                     |
                     ^
    }
}
"#,
    ));
}

#[test]
fn array_indexed_access_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum struct Foo {
    int foo;
}

void foo() {
    Foo baz[1];
    baz[0].foo = 1;
            |
            ^
}
"#,
    ));
}

#[test]
fn array_indexed_access_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum struct Foo {
    int foo;
}

void foo() {
    Foo baz[1][1] = {};
    baz[0][0].foo = 1;
               |
               ^
}
"#,
    ));
}
