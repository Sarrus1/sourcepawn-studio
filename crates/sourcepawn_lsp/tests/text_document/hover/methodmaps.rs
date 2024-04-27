use insta::assert_json_snapshot;
use sourcepawn_lsp::fixture::hover;

#[test]
fn methodmap_1() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
/**
 * The Foo methodmap.
 */
methodmap Foo {}
           |
           ^
"#,
    ));
}

#[test]
fn methodmap_2() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
methodmap Foo {}

/**
 * The Bar methodmap.
 */
methodmap Bar < Foo {}
           |
           ^
"#,
    ));
}

#[test]
fn methodmap_constructor_1() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
methodmap Foo {
    /**
     * The Foo constructor.
     */
    public Foo() {}
            |
            ^
}
"#,
    ));
}

#[test]
fn methodmap_method_1() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
methodmap Foo {
    /**
     * The foo method.
     */
    public void foo() {}
                 |
                 ^
}
"#,
    ));
}

#[test]
fn methodmap_property_1() {
    assert_json_snapshot!(hover(
        r#"
%! main.sp
methodmap Foo {
    /**
     * The foo property.
     */
    property int foo {}
                  |
                  ^
}
"#,
    ));
}
