use insta::assert_json_snapshot;
use sourcepawn_studio::fixture::goto_definition;

#[test]
fn for_loop_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
void foo() {
    for (int bar; bar < 10; bar++) {
              |
              ^
    }
}
"#,
    ));
}

#[test]
fn for_loop_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
void foo() {
    for (int bar; bar < 10; bar++) {
        bar = 5;
         |
         ^
    }
}
"#,
    ));
}

#[test]
fn for_loop_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
void foo() {
    for (int bar; bar < 10; bar++) {
                   |
                   ^
    }
}
"#,
    ));
}

#[test]
fn for_loop_4() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
void foo() {
    int bar, baz;
    for (bar = 0, baz = 0; bar < 10; bar++) {
          |
          ^
    }
}
"#,
    ));
}

#[test]
fn for_loop_5() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum struct Foo {
    int bar;
    void Get(int foo) {}
}

int main() {
    Foo foo;
    for (int i = 1; --i >= 0;) {
        foo.Get(1);
             |
             ^
    }
    foo.Get(1);
}
"#,
    ));
}
