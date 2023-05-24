use std::fs;

use lazy_static::lazy_static;
use lsp_types::{CompletionItem, CompletionItemKind, CompletionList, CompletionParams, Url};
use regex::Regex;

use super::FeatureRequest;

pub(super) struct IncludeStatement {
    /// Text inside of the include statement, excluding the traling quotation marks or chevrons.
    text: String,

    /// Whether the include uses `<>` or `""`.
    use_chevron: bool,
}

/// Determine whether the current prefix line is the beginning of an include statement.
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

    match_.map(|match_| IncludeStatement {
        text: match_.get(1).unwrap().as_str().to_string(),
        use_chevron,
    })
}

/// Generate a [CompletionList](lsp_types::CompletionList) from an [IncludeStatement](IncludeStatement).
///
/// # Arguments
///
/// * `sub_line` - Sub line to process.
pub(super) fn get_include_completions(
    request: FeatureRequest<CompletionParams>,
    include_st: IncludeStatement,
) -> Option<CompletionList> {
    let include_paths = request
        .store
        .environment
        .options
        .get_all_possible_include_folders();

    let mut inc_uri_folders: Vec<Url> = vec![];
    for inc_path in include_paths {
        if let Ok(inc_uri) = Url::from_file_path(inc_path) {
            inc_uri_folders.push(inc_uri);
        }
    }

    let mut items = vec![];
    get_include_file_completions(request, &include_st, &inc_uri_folders, &mut items);
    get_include_folder_completions(&include_st, &inc_uri_folders, &mut items);

    Some(CompletionList {
        items,
        ..Default::default()
    })
}

/// Mutates a vector of [CompletionItem](lsp_types::CompletionItem) to push include file completions
/// to it.
///
/// # Arguments
///
/// * `request` - Associated [FeatureRequest<CompletionParams>](FeatureRequest<CompletionParams>).
/// * `include_st` - [IncludeStatement] to base the request off of.
/// * `inc_uri_folders` - Vector of folder [uris](lsp_types::Url) into which to look for includes.
/// * `items` - Vector of [CompletionItem](lsp_types::CompletionItem) to mutate.
fn get_include_file_completions(
    request: FeatureRequest<CompletionParams>,
    include_st: &IncludeStatement,
    inc_uri_folders: &[Url],
    items: &mut Vec<CompletionItem>,
) -> Option<()> {
    lazy_static! {
        static ref RE1: Regex = Regex::new(r"(?P<a>(?:[^'</]+/)+)+").unwrap();
    }

    // Extract everything that has already been typed in the statement.
    let typed_path = RE1.replace(&include_st.text, "$a").to_string();

    for inc_uri in request.store.documents.keys() {
        for inc_uri_folder in inc_uri_folders.iter() {
            if !inc_uri
                .to_string()
                .contains(&format!("{}/{}", inc_uri_folder, typed_path))
            {
                continue;
            }
            if let Ok(inc_path) = inc_uri.to_file_path() {
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
                    // FIXME: This could be fixed programmatically to account for other editors.
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

/// Mutates a vector of [CompletionItem](lsp_types::CompletionItem) to push include folder completions
/// to it.
///
/// # Arguments
///
/// * `include_st` - [IncludeStatement] to base the request off of.
/// * `inc_uri_folders` - Vector of folder [uris](lsp_types::Url) into which to look for includes.
/// * `items` - Vector of [CompletionItem](lsp_types::CompletionItem) to mutate.
fn get_include_folder_completions(
    include_st: &IncludeStatement,
    inc_uri_folders: &[Url],
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
