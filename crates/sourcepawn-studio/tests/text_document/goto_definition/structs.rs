use insta::assert_json_snapshot;
use sourcepawn_studio::fixture::goto_definition;

#[test]
fn struct_definition_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
struct Plugin
          |
          ^
{
   public const char[] name;        /**< Plugin Name */
   public const char[] description; /**< Plugin Description */
   public const char[] author;      /**< Plugin Author */
   public const char[] version;     /**< Plugin Version */
   public const char[] url;         /**< Plugin URL */
};
"#,
    ));
}

#[test]
fn struct_field_definition_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
struct Plugin
{
   public const char[] name;        /**< Plugin Name */
                         |
                         ^
   public const char[] description; /**< Plugin Description */
   public const char[] author;      /**< Plugin Author */
   public const char[] version;     /**< Plugin Version */
   public const char[] url;         /**< Plugin URL */
};
"#,
    ));
}

#[test]
fn struct_field_definition_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
struct Plugin
{
   public const char[] name;        /**< Plugin Name */
   public const char[] description; /**< Plugin Description */
   public const char[] author;      /**< Plugin Author */
                         |
                         ^
   public const char[] version;     /**< Plugin Version */
   public const char[] url;         /**< Plugin URL */
};
"#,
    ));
}

#[test]
fn struct_declaration_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
struct Plugin
{
   public const char[] name;        /**< Plugin Name */
   public const char[] description; /**< Plugin Description */
   public const char[] author;      /**< Plugin Author */
   public const char[] version;     /**< Plugin Version */
   public const char[] url;         /**< Plugin URL */
};

public Plugin myinfo = 
          |
          ^
{
   name = "name",
   author = "author",
   description = "description",
   version = "version",
   url = "url"
};
"#,
    ));
}

#[test]
fn struct_declaration_2() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
struct Plugin
{
   public const char[] name;        /**< Plugin Name */
   public const char[] description; /**< Plugin Description */
   public const char[] author;      /**< Plugin Author */
   public const char[] version;     /**< Plugin Version */
   public const char[] url;         /**< Plugin URL */
};

public Plugin myinfo = 
                 |
                 ^
{
   name = "name",
   author = "author",
   description = "description",
   version = "version",
   url = "url"
};
"#,
    ));
}

#[test]
fn struct_field_declaration_1() {
    assert_json_snapshot!(goto_definition(
        r#"
%! main.sp
struct Plugin
{
   public const char[] name;        /**< Plugin Name */
   public const char[] description; /**< Plugin Description */
   public const char[] author;      /**< Plugin Author */
   public const char[] version;     /**< Plugin Version */
   public const char[] url;         /**< Plugin URL */
};

public Plugin myinfo = 
{
   name = "name",
      |
      ^
   author = "author",
   description = "description",
   version = "version",
   url = "url"
};
"#,
    ));
}
