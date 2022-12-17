use std::fs;

use lazy_static::lazy_static;
use lsp_types::{CompletionItem, CompletionItemKind, CompletionList, CompletionParams, Url};
use regex::Regex;

use super::FeatureRequest;

pub(super) struct IncludeStatement {
    text: String,
    use_chevron: bool,
}

pub(super) fn is_include_statement(sub_line: &str) -> Option<IncludeStatement> {
    lazy_static! {
        static ref RE1: Regex = Regex::new(r"^\s*#\s*include\s*(?:<([^>]*)>?)").unwrap();
        static ref RE2: Regex = Regex::new("^\\s*#\\s*include\\s*(?:\"([^\"]*)\"?)").unwrap();
    }

    let mut match_ = RE1.captures(sub_line);
    let mut use_chevron = true;
    if match_.is_none() {
        match_ = RE2.captures(sub_line);
        use_chevron = false;
    }

    match_.map(|match_| IncludeStatement {
        text: match_.get(1).unwrap().as_str().to_string(),
        use_chevron,
    })
}

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

    let mut items: Vec<CompletionItem> = vec![];
    get_include_file_completions(request, &include_st, &inc_uri_folders, &mut items);
    get_include_folder_completions(&include_st, &inc_uri_folders, &mut items);

    Some(CompletionList {
        items,
        ..Default::default()
    })
}

fn get_include_file_completions(
    request: FeatureRequest<CompletionParams>,
    include_st: &IncludeStatement,
    inc_uri_folders: &[Url],
    items: &mut Vec<CompletionItem>,
) {
    lazy_static! {
        static ref RE1: Regex = Regex::new(r"(?P<a>(?:[^'</]+/)+)+").unwrap();
    }

    let prev_path = RE1.replace(&include_st.text, "$a").to_string();

    for inc_uri in request.store.documents.keys() {
        for inc_uri_folder in inc_uri_folders.iter() {
            if inc_uri
                .to_string()
                .contains(&format!("{}/{}", inc_uri_folder, prev_path))
            {
                if let Ok(inc_path) = inc_uri.to_file_path() {
                    let parent_folder = inc_uri_folder
                        .to_file_path()
                        .unwrap()
                        .join(&include_st.text);
                    if parent_folder
                        .to_str()
                        .unwrap()
                        .contains(inc_path.parent().unwrap().to_str().unwrap())
                    {
                        let label = inc_path
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
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
                            detail: Some(inc_path.to_str().unwrap().to_string()),
                            ..Default::default()
                        });
                        break;
                    }
                }
            }
        }
    }
}

fn get_include_folder_completions(
    include_st: &IncludeStatement,
    inc_uri_folders: &[Url],
    items: &mut Vec<CompletionItem>,
) {
    for inc_uri_folder in inc_uri_folders.iter() {
        let inc_folder_path = inc_uri_folder.to_file_path().unwrap();

        for path in fs::read_dir(inc_folder_path.clone())
            .unwrap()
            .into_iter()
            .flatten()
        {
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
