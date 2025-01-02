use base_db::FilePosition;
use hir::{DefResolution, Semantics};
use ide_db::RootDatabase;
use lazy_static::lazy_static;
use line_index::TextRange;
use regex::Regex;
use smol_str::ToSmolStr;
use syntax::TSKind;

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
/// * `pos` - [`FilePosition`] where the completion was triggered.
/// * `source` - The preprocessed text of the document.
pub(super) fn get_doc_completion(
    db: &RootDatabase,
    pos: FilePosition,
    source: &str,
) -> Option<Vec<crate::CompletionItem>> {
    let sema = &Semantics::new(db);
    let tree = sema.parse(pos.file_id);
    let mut offset_below = find_first_non_ws_after_newline(source, pos.raw_offset_usize())?;
    let mut node = tree
        .root_node()
        .descendant_for_byte_range(offset_below, offset_below)
        .or_else(|| {
            // Hack:
            // Sometimes the lookup fails because of a type that is too short.
            // It also always fails for methodmap properties.
            // This is a hacky workaround to try again.
            offset_below += 1;
            tree.root_node()
                .descendant_for_byte_range(offset_below, offset_below)
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
    let def = sema.find_name_def(pos.file_id, &name)?;
    let tab_str = tab_str(&source[find_first_newline(source, pos.raw_offset_usize())?..])?;
    let res = match def {
        DefResolution::Function(it) => {
            snippet_builder(it.parameters(db), it.type_ref(db), &tab_str)
        }
        DefResolution::Typedef(it) => {
            snippet_builder(it.parameters(db), it.return_type(db).into(), &tab_str)
        }
        DefResolution::Functag(it) => {
            snippet_builder(it.parameters(db), it.return_type(db), &tab_str)
        }
        DefResolution::Enum(_)
        | DefResolution::Methodmap(_)
        | DefResolution::EnumStruct(_)
        | DefResolution::Typeset(_)
        | DefResolution::Funcenum(_)
        | DefResolution::Property(_) => snippet_builder(vec![], None, &tab_str),
        _ => unreachable!(),
    };

    Some(vec![CompletionItem {
        label: "Insert documentation".to_smolstr(),
        kind: crate::CompletionKind::Snippet,
        filter_text: "/*".to_string().into(),
        insert_text: Some(res.clone()),
        text_edit: Some((TextRange::at(pos.raw_offset().into(), 0.into()), res)),
        ..Default::default()
    }])
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
        res.push_str(tab_str);
        res.push_str(" * ");
        res.push_str(line);
    }
    res.push('\n');
    res.push_str(tab_str);
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

fn find_first_non_ws_after_newline(text: &str, raw_offset: usize) -> Option<usize> {
    if raw_offset >= text.len() {
        return None;
    }
    let newline_pos = text[raw_offset..].find('\n')?;
    let after_newline_offset = raw_offset + newline_pos + 1;
    text[after_newline_offset..]
        .char_indices()
        .find(|&(_, c)| !c.is_whitespace())
        .map(|(i, _)| after_newline_offset + i)
}

fn find_first_newline(text: &str, raw_offset: usize) -> Option<usize> {
    if raw_offset >= text.len() {
        return None;
    }
    text[raw_offset..].find('\n')
}
