use base_db::SourceDatabaseExt;
use fxhash::FxHashSet;
use ide_db::RootDatabase;
use itertools::Itertools;
use lazy_static::lazy_static;
use lsp_types::Url;
use paths::AbsPathBuf;
use regex::Regex;
use smol_str::ToSmolStr;
use std::panic::AssertUnwindSafe;
use vfs::FileId;

use crate::completion::item::CompletionKind;

#[derive(Debug, Clone)]
pub(super) struct IncludeStatement {
    /// Text inside of the include statement, excluding the traling quotation marks or chevrons.
    text: String,

    /// Whether the include uses `<>` or `""`.
    use_chevron: bool,

    /// Whether the completion should append a closing element to the path string.
    should_append_closing_element: bool,
}

/// Check whether the current prefix line is the beginning of an include statement.
/// Return None if it's not, and an [IncludeStatement] object if it is.
///
/// # Arguments
///
/// * `pre_line` - Prefix line to process.
/// * `post_line` - Postfix line to process.
pub(super) fn is_include_statement(pre_line: &str, post_line: &str) -> Option<IncludeStatement> {
    lazy_static! {
        static ref RE1: Regex = Regex::new(r"^\s*#\s*include\s*(?:<([^>]*)(>)?)").unwrap();
        static ref RE2: Regex = Regex::new("^\\s*#\\s*include\\s*(?:\"([^\"]*)(\")?)").unwrap();
    }

    let mut match_ = RE1.captures(pre_line);
    let mut use_chevron = true;
    if match_.is_none() {
        match_ = RE2.captures(pre_line);
        use_chevron = false;
    }

    let should_append_closing_element = !{
        if use_chevron {
            post_line.trim().starts_with('>')
        } else {
            post_line.trim().starts_with('"')
        }
    };

    match_.and_then(|match_| {
        IncludeStatement {
            text: match_.get(1)?.as_str().to_string(),
            use_chevron,
            should_append_closing_element,
        }
        .into()
    })
}

/// Generate a [CompletionList](lsp_types::CompletionList) from an [IncludeStatement](IncludeStatement).
pub(super) fn get_include_completions(
    db: &RootDatabase,
    include_st: IncludeStatement,
    file_id: FileId,
    mut include_directories: Vec<AbsPathBuf>,
    file_id_to_url: AssertUnwindSafe<&dyn Fn(FileId) -> Url>,
) -> Option<Vec<crate::CompletionItem>> {
    let path: AbsPathBuf = file_id_to_url(file_id)
        .to_file_path()
        .ok()?
        .try_into()
        .ok()?;
    let parent_folder: AbsPathBuf = path.parent()?.to_path_buf();
    let mut known_paths: FxHashSet<AbsPathBuf> =
        FxHashSet::from_iter(db.known_files().iter().flat_map(|(file_id, _)| {
            file_id_to_url(*file_id)
                .to_file_path()
                .ok()?
                .try_into()
                .ok()
        }));
    known_paths.remove(&path);

    if !include_st.use_chevron {
        include_directories.push(parent_folder);
    }

    lazy_static! {
        static ref RE1: Regex = Regex::new(r"(?P<a>(?:[^'</]+/)+)+").unwrap();
    }

    // Extract everything that has already been typed in the statement.
    let typed_path = RE1.replace(&include_st.text, "$a").to_string();

    let include_directories_hash = FxHashSet::from_iter(
        include_directories
            .into_iter()
            .map(|it| it.join(&typed_path)),
    );
    let completions = known_paths
        .iter()
        .filter_map(|it| {
            for path in include_directories_hash.iter() {
                if let Some(stripped) = it.strip_prefix(path) {
                    return Some((
                        stripped.components().next()?.as_os_str().to_str()?,
                        stripped.components().count() > 1,
                    ));
                }
            }
            None
        })
        .sorted()
        .dedup()
        .collect_vec();

    let closing_element = if include_st.should_append_closing_element {
        if include_st.use_chevron {
            ">"
        } else {
            "\""
        }
    } else {
        ""
    };

    let items = completions
        .into_iter()
        .map(|(it, is_dir)| {
            let mut insert_text = it.replace(".inc", "");
            insert_text.push_str(closing_element);
            crate::CompletionItem {
                label: it.to_smolstr(),
                kind: if is_dir {
                    CompletionKind::Directory
                } else {
                    CompletionKind::File
                },
                insert_text: Some(insert_text),
                detail: Some(it.to_string()),
                ..Default::default()
            }
        })
        .collect_vec();

    Some(items)
}
