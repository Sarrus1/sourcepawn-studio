use insta::assert_json_snapshot;
use sourcepawn_lsp::fixture::goto_definition;

#[test]
fn methodmap_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {}
           |
           ^
"#,
    ));
}

#[test]
fn methodmap_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {}

methodmap Bar < Foo {}
                 |
                 ^
"#,
    ));
}

#[test]
fn methodmap_method_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    public void Foo() {}
                 |
                 ^
}
"#,
    ));
}

#[test]
fn methodmap_method_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    public void Foo(int foo) {}
                         |
                         ^
}
"#,
    ));
}

#[test]
fn methodmap_method_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    public void Foo(int foo) {
        foo += 1;
         |
         ^
    }
}
"#,
    ));
}

#[test]
fn methodmap_method_4() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    public void Foo() {}
    public void Bar() {
        this.Foo();
          |
          ^
    }
}
"#,
    ));
}

#[test]
fn methodmap_method_5() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    public void Foo() {}
    public void Bar() {
        this.Foo();
              |
              ^
    }
}
"#,
    ));
}

#[test]
fn methodmap_constructor_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    public Foo() {}
            |
            ^
}
"#,
    ));
}

#[test]
fn methodmap_constructor_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    public Foo(int foo) {}
                    |
                    ^
}
"#,
    ));
}

#[test]
fn methodmap_constructor_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    public Foo(int foo) {
        foo += 1;
         |
         ^
    }
}
"#,
    ));
}

#[test]
fn methodmap_destructor_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    public ~Foo() {}
             |
             ^
}
"#,
    ));
}
