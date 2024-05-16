use hir::{DefResolution, Semantics};
use ide_db::RootDatabase;
use lazy_static::lazy_static;
use lsp_types::{Position, Range};
use regex::Regex;
use smol_str::ToSmolStr;
use syntax::TSKind;
use tree_sitter::Point;
use vfs::FileId;

use crate::CompletionItem;

/// Check whether the current prefix line is the beginning of a doc comment.
///
/// # Arguments
///
/// * `pre_line` - Prefix line to process.
/// * `post_line` - Postfix line to process.
pub(super) fn is_documentation_start(pre_line: &str, post_line: &str) -> bool {
    pre_line.ends_with("/*") && post_line.is_empty()
}

/// Generate a doc completion for the node below `point`, if possible.
///
/// # Arguments
///
/// * `db` - [`RootDatabase`] instance.
/// * `point` - [`Point`] where the completion was triggered.
/// * `file_id` - [`FileId`] where the completion was triggered.
pub(super) fn get_doc_completion(
    db: &RootDatabase,
    point: Point,
    file_id: FileId,
) -> Option<Vec<crate::CompletionItem>> {
    let sema = &Semantics::new(db);
    let tree = sema.parse(file_id);
    let mut point_below = Point::new(point.row.saturating_add(1), point.column);
    let mut node = tree
        .root_node()
        .descendant_for_point_range(point_below, point_below)
        .or_else(|| {
            // Hack:
            // Sometimes the lookup fails because of a type that is too short.
            // It also always fails for methodmap properties.
            // This is a hacky workaround to try again.
            point_below.column = point_below.column.saturating_add(1);
            tree.root_node()
                .descendant_for_point_range(point_below, point_below)
        })?;

    while let Some(parent) = node.parent() {
        if matches!(
            TSKind::from(parent),
            TSKind::function_definition
                | TSKind::function_declaration
                | TSKind::methodmap_method_constructor
                | TSKind::methodmap_method_destructor
                | TSKind::methodmap_method
                | TSKind::methodmap_property_getter
                | TSKind::methodmap_property_setter
                | TSKind::enum_struct_method
                | TSKind::r#enum
                | TSKind::enum_struct
                | TSKind::methodmap
                | TSKind::methodmap_property
                | TSKind::typeset
                | TSKind::typedef
                | TSKind::funcenum
                | TSKind::functag
        ) {
            break;
        }
        node = parent;
    }
    node = node.parent()?;
    let name = node.child_by_field_name("name")?;
    let def = sema.find_name_def(file_id, &name)?;
    let tab_str = tab_str(
        sema.preprocessed_text(file_id)
            .lines()
            .nth(point_below.row)?,
    )?;
    let res = match def {
        DefResolution::Function(it) => {
            snippet_builder(it.parameters(db), it.type_ref(db), &tab_str)
        }
        DefResolution::Typedef(it) => {
            snippet_builder(it.parameters(db), it.return_type(db).into(), &tab_str)
        }
        DefResolution::Functag(it) => {
            snippet_builder(it.parameters(db), it.return_type(db).into(), &tab_str)
        }
        DefResolution::Enum(_)
        | DefResolution::Methodmap(_)
        | DefResolution::EnumStruct(_)
        | DefResolution::Typeset(_)
        | DefResolution::Funcenum(_)
        | DefResolution::Property(_) => snippet_builder(vec![], None, &tab_str),
        _ => unreachable!(),
    };
    return Some(vec![CompletionItem {
        label: "Insert documentation".to_smolstr(),
        kind: crate::CompletionKind::Snippet,
        filter_text: "/*".to_string().into(),
        insert_text: Some(res.clone()),
        text_edit: Some((
            Range::new(
                Position::new(point.row as u32, 0),
                Position::new(point.row as u32, 0),
            ),
            res,
        )),
        ..Default::default()
    }]);
}

fn snippet_builder(params: Vec<String>, ret_type: Option<String>, tab_str: &str) -> String {
    let mut buf = Vec::new();
    buf.push("${1:Description}".to_string());
    let mut max = 0;
    params.iter().for_each(|it| {
        if it.len() > max {
            max = it.len()
        }
    });
    for (i, param) in params.iter().enumerate() {
        buf.push(format!(
            "@param {}{}${{{}:Parameter description}}",
            param,
            " ".repeat(max - param.len() + 2),
            i + 2
        ))
    }
    if let Some(ret) = ret_type {
        if ret != "void" {
            buf.push(format!(
                "@return  ${{{}:Return description}}",
                params.len() + 2
            ));
        }
    }
    let mut res = format!("{}/**", tab_str);
    for line in buf.iter() {
        res.push('\n');
        res.push_str(&tab_str);
        res.push_str(" * ");
        res.push_str(&line);
    }
    res.push('\n');
    res.push_str(&tab_str);
    res.push_str(" */");

    res
}

fn tab_str(pre_line: &str) -> Option<String> {
    lazy_static! {
        pub static ref WS_REGEX: Regex = Regex::new(r"^(\s*)").unwrap();
    }
    WS_REGEX
        .captures(pre_line)?
        .get(0)?
        .as_str()
        .to_string()
        .into()
}
