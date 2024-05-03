use base_db::{SourceDatabase, SourceDatabaseExt};
use fxhash::FxHashSet;
use hir::Semantics;
use ide_db::{RootDatabase, SymbolKind};
use itertools::Itertools;
use lazy_static::lazy_static;
use lsp_types::{CompletionItem, CompletionItemKind, CompletionList, Url};
use paths::AbsPathBuf;
use regex::Regex;
use smol_str::ToSmolStr;
use std::{fs, panic::AssertUnwindSafe, path::PathBuf};
use vfs::FileId;

#[derive(Debug, Clone)]
pub(super) struct IncludeStatement {
    /// Text inside of the include statement, excluding the traling quotation marks or chevrons.
    text: String,

    /// Whether the include uses `<>` or `""`.
    use_chevron: bool,
}

/// Check whether the current prefix line is the beginning of an include statement.
/// Return None if it's not, and an [IncludeStatement] object if it is.
///
/// # Arguments
///
/// * `pre_line` - Prefix line to process.
pub(super) fn is_include_statement(pre_line: &str) -> Option<IncludeStatement> {
    lazy_static! {
        static ref RE1: Regex = Regex::new(r"^\s*#\s*include\s*(?:<([^>]*)>?)").unwrap();
        static ref RE2: Regex = Regex::new("^\\s*#\\s*include\\s*(?:\"([^\"]*)\"?)").unwrap();
    }

    let mut match_ = RE1.captures(pre_line);
    let mut use_chevron = true;
    if match_.is_none() {
        match_ = RE2.captures(pre_line);
        use_chevron = false;
    }

    match_.and_then(|match_| {
        IncludeStatement {
            text: match_.get(1)?.as_str().to_string(),
            use_chevron,
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
    let mut known_paths: FxHashSet<AbsPathBuf> = FxHashSet::default();
    // known_uris.insert(parent_folder_uri);
    known_paths.extend(db.known_files().iter().flat_map(|(file_id, _)| {
        file_id_to_url(*file_id)
            .to_file_path()
            .ok()?
            .try_into()
            .ok()
    }));

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
        .filter(|it| {
            let Some(parent) = it.parent() else {
                return false;
            };
            include_directories_hash.contains(parent)
        })
        .collect_vec();

    let items = completions
        .into_iter()
        .flat_map(|it| {
            if it.is_dir() {
                Some(crate::CompletionItem {
                    label: it.file_name()?.to_str()?.to_smolstr(),
                    kind: SymbolKind::Directory,
                    detail: Some(it.to_string()),
                    documentation: None,
                    deprecated: false,
                    trigger_call_info: false,
                })
            } else {
                Some(crate::CompletionItem {
                    label: it.file_name()?.to_str()?.to_smolstr(),
                    kind: SymbolKind::File,
                    detail: Some(it.to_string()),
                    documentation: None,
                    deprecated: false,
                    trigger_call_info: false,
                })
            }
        })
        .collect_vec();

    // get_include_file_completions(store, &include_st, &known_paths, &mut items);
    // get_include_folder_completions(&include_st, &known_paths, &mut items);

    Some(items)
}

/*
/// Mutates a set of [CompletionItem](lsp_types::CompletionItem) to push include file completions
/// to it.
///
/// # Arguments
///
/// * `store` -
/// * `include_st` - [IncludeStatement] to base the request off of.
/// * `inc_uri_folders` - Vector of folder [uris](lsp_types::Url) into which to look for includes.
/// * `items` - Vector of [CompletionItem](lsp_types::CompletionItem) to mutate.
fn get_include_file_completions(
    store: &Store,
    include_st: &IncludeStatement,
    inc_uri_folders: &FxHashSet<Url>,
    items: &mut Vec<CompletionItem>,
) -> Option<()> {
    lazy_static! {
        static ref RE1: Regex = Regex::new(r"(?P<a>(?:[^'</]+/)+)+").unwrap();
    }

    // Extract everything that has already been typed in the statement.
    let typed_path = RE1.replace(&include_st.text, "$a").to_string();

    for document in store.documents.values() {
        for inc_uri_folder in inc_uri_folders.iter() {
            if !document
                .uri
                .to_string()
                .contains(&format!("{}/{}", inc_uri_folder, typed_path))
            {
                continue;
            }
            if let Ok(inc_path) = document.uri.to_file_path() {
                let parent_folder = inc_uri_folder
                    .to_file_path()
                    .unwrap()
                    .join(&include_st.text);
                if !parent_folder
                    .to_str()?
                    .contains(inc_path.parent()?.to_str()?)
                {
                    continue;
                }
                let label = inc_path
                    .file_name()?
                    .to_str()?
                    .to_string()
                    .replace(".inc", "");
                let mut trail = ">";
                if !include_st.use_chevron {
                    // Don't insert anything as VSCode already autocompletes the second ".
                    // TODO: This could be fixed programmatically to account for other editors.
                    trail = "";
                }
                items.push(CompletionItem {
                    label: label.clone(),
                    insert_text: Some(format!("{}{}", label, trail)),
                    kind: Some(CompletionItemKind::FILE),
                    detail: Some(inc_path.to_str()?.to_string()),
                    ..Default::default()
                });
                break;
            }
        }
    }

    Some(())
}

/// Mutates a set of [CompletionItem](lsp_types::CompletionItem) to push include folder completions
/// to it.
///
/// # Arguments
///
/// * `include_st` - [IncludeStatement] to base the request off of.
/// * `inc_uri_folders` - Vector of folder [uris](lsp_types::Url) into which to look for includes.
/// * `items` - Vector of [CompletionItem](lsp_types::CompletionItem) to mutate.
fn get_include_folder_completions(
    include_st: &IncludeStatement,
    inc_uri_folders: &FxHashSet<Url>,
    items: &mut Vec<CompletionItem>,
) {
    for inc_uri_folder in inc_uri_folders.iter() {
        let inc_folder_path = inc_uri_folder.to_file_path().unwrap();
        let paths = fs::read_dir(inc_folder_path.clone()).unwrap().flatten();

        for path in paths {
            let path = path.path();
            if !path.is_dir() {
                continue;
            }
            let tmp_path = inc_folder_path.join(&include_st.text);
            if !path.to_str().unwrap().contains(tmp_path.to_str().unwrap()) {
                continue;
            }
            let label = path.file_name().unwrap().to_str().unwrap().to_string();
            items.push(CompletionItem {
                label: label.clone(),
                kind: Some(CompletionItemKind::FOLDER),
                detail: Some(path.to_str().unwrap().to_string()),
                ..Default::default()
            });
        }
    }
}

 */
