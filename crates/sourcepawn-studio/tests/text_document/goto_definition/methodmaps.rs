use insta::assert_json_snapshot;
use sourcepawn_studio::fixture::goto_definition;

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
fn methodmap_method_6() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    public void Foo() {}
}

Foo foo;

void main() {
    foo.Foo();
         |
         ^
}
"#,
    ));
}

#[test]
fn methodmap_method_7() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    public void Foo() {int bar}
}

Foo foo;

void main() {
    int bar;
    foo.Foo(bar);
             |
             ^
}
"#,
    ));
}

#[test]
fn methodmap_method_8() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
#include "bar.sp"
methodmap Foo < Bar {
    public void foo() {}
                 |
                 ^
}

%! bar.sp
methodmap Bar {
    public void Bar1() {}
    public void Bar2() {}
    public void Bar3() {}
}
"#,
    ));
}

#[test]
fn methodmap_native_method_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    public native int Foo();
                       |
                       ^
}
"#,
    ));
}

#[test]
fn methodmap_native_method_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    public native int Foo(int foo);
                               |
                               ^
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

#[test]
fn methodmap_property_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    property int Foo {}
                  |
                  ^
}
"#,
    ));
}

#[test]
fn methodmap_property_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
enum Bar {
    Baz
}

methodmap Foo {
    property Bar Foo {}
              |
              ^
}
"#,
    ));
}

#[test]
fn methodmap_property_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    property int Foo {}
}

Foo foo;

void main() {
    foo.Foo;
         |
         ^
}
"#,
    ));
}

#[test]
fn methodmap_property_getter_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    property int Foo {
        public get() { return 1; }
                |
                ^
    }
}
"#,
    ));
}

#[test]
fn methodmap_property_getter_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
int foo;
methodmap Foo {
    property int Foo {
        public get() { 
            return foo;
                    |
                    ^
        }
    }
}
"#,
    ));
}

#[test]
fn methodmap_property_getter_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    property int Foo {
        public get() { 
            int foo;
            return foo;
                    |
                    ^
        }
    }
}
"#,
    ));
}

#[test]
fn methodmap_property_setter_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    property int Foo {
        public set(int foo) {}
                |
                ^
    }
}
"#,
    ));
}

#[test]
fn methodmap_property_setter_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    property int Foo {
        public set(int foo) {}
                        |
                        ^
    }
}
"#,
    ));
}

#[test]
fn methodmap_property_setter_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    property int Foo {
        public set(int foo) {
            foo += 1;
             |
             ^
        }
    }
}
"#,
    ));
}

#[test]
fn methodmap_property_setter_4() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    property int Foo {
        public set(int foo) {
            int foo;
            foo += 1;
             |
             ^
        }
    }
}
"#,
    ));
}

#[test]
fn methodmap_property_native_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    property int Foo {
        public native get();
                       |
                       ^
        }
    }
}
"#,
    ));
}

#[test]
fn methodmap_property_native_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    property int Foo {
        public native set(int foo);
                       |
                       ^
        }
    }
}
"#,
    ));
}

#[test]
fn methodmap_property_native_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    property int Foo {
        public native set(int foo);
                               |
                               ^
        }
    }
}
"#,
    ));
}

#[test]
fn methodmap_new_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    public Foo() {}
    public void bar() {}
}

void main() {
    Foo foo = new Foo();
    foo.bar();
         |
         ^
}
"#,
    ));
}

#[test]
fn methodmap_new_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    public Foo() {}
}

void main() {
    new Foo();
         |
         ^
}
"#,
    ));
}

#[test]
fn methodmap_inherit_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    public void Foo() {}
    public void Foo2() {}
}
methodmap Bar < Foo {
    public void Bar() {}
}
Bar bar;
void main() {
    bar.Foo2();
         |
         ^
}
"#,
    ));
}

#[test]
fn methodmap_inherit_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    property int Foo
    {
        public get() {}
        public set(int value) {}
    }
    property int Bar
    {
        public get() {}
        public set(int value) {}
    }
}
methodmap Bar < Foo {
    public void Bar2() {}
}
Bar bar;
void main() {
    bar.Bar;
         |
         ^
}
"#,
    ));
}

#[test]
fn methodmap_inherit_3() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    property int Foo
    {
        public get() {}
        public set(int value) {}
    }
    property int Bar
    {
        public get() {}
        public set(int value) {}
    }
}
methodmap Bar < Foo {
    property int Bar
    {
        public get() {}
        public set(int value) {}
    }
}
Bar bar;
void main() {
    bar.Bar;
         |
         ^
}
"#,
    ));
}

#[test]
fn methodmap_inherit_4() {
    assert_json_snapshot!(goto_definition(
        r#"
%! foo.sp
methodmap Foo {
    property int Foo1
    {
        public get() {}
        public set(int value) {}
    }
    property int Foo2
    {
        public get() {}
        public set(int value) {}
    }
    property int Foo3
    {
        public get() {}
        public set(int value) {}
    }
    property int Bar
    {
        public get() {}
        public set(int value) {}
    }
}

%! bar.sp
#include "foo.sp"
methodmap Bar < Foo {
    property int Bar
    {
        public get() {}
        public set(int value) {}
    }
}

%! main.sp
#include "bar.sp"

Bar bar;
void main() {
    bar.Bar;
         |
         ^
}
"#,
    ));
}

#[test]
fn methodmap_complex_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
methodmap Foo {
    public native Foo Bar();
                       |
                       ^
    public native int Baz();
    property int Qux {
        public native get();
    }
}
"#,
    ));
}
