use insta::assert_json_snapshot;
use sourcepawn_studio::fixture::complete;

#[test]
fn macro_positive_offset_global_scope() {
    assert_json_snapshot!(complete(
        r#"
%! main.sp
#define FOOOOO 1
int foo = FOOOOO;

|
^"#,
        None
    ));
}

#[test]
fn macro_negative_offset_global_scope() {
    assert_json_snapshot!(complete(
        r#"
%! main.sp
#define F 10000000
int foo = F;

|
^"#,
        None
    ));
}
