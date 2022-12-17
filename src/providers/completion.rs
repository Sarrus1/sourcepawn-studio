use std::fs;

use lazy_static::lazy_static;
use lsp_types::{CompletionItem, CompletionItemKind, CompletionList, CompletionParams, Url};
use regex::Regex;

use crate::spitem::get_all_items;

use self::{
    context::is_method_call,
    getters::{get_method_completions, get_non_method_completions},
};

use super::FeatureRequest;

mod context;
mod getters;
mod matchtoken;

pub fn provide_completions(request: FeatureRequest<CompletionParams>) -> Option<CompletionList> {
    let document = request.store.get(&request.uri)?;
    let all_items = get_all_items(&request.store)?;
    let position = request.params.text_document_position.position;
    let line = document.line(position.line)?;
    let sub_line: String = line.chars().take(position.character as usize).collect();

    lazy_static! {
        static ref RE1: Regex = Regex::new(r"^\s*#\s*include\s*(?:<([^>]*)>?)").unwrap();
        static ref RE2: Regex = Regex::new("^\\s*#\\s*include\\s*(?:\"([^\"]*)\"?)").unwrap();
    }

    let mut match_ = RE1.captures(sub_line.as_str());
    let mut use_chevron = true;
    if match_.is_none() {
        match_ = RE2.captures(sub_line.as_str());
        use_chevron = false;
    }
    if let Some(match_) = match_ {
        return get_include_completions(request, use_chevron, match_.get(1).unwrap().as_str());
    }

    if !is_method_call(line, position) {
        return get_non_method_completions(all_items, request.params);
    }

    get_method_completions(all_items, line, position, request)
}

fn get_include_completions(
    request: FeatureRequest<CompletionParams>,
    use_chevron: bool,
    text: &str,
) -> Option<CompletionList> {
    let include_paths = request
        .store
        .environment
        .options
        .get_all_possible_include_folder();

    let mut inc_uri_folders: Vec<Url> = vec![];
    for inc_path in include_paths {
        if let Ok(inc_uri) = Url::from_file_path(inc_path) {
            inc_uri_folders.push(inc_uri);
        }
    }

    lazy_static! {
        static ref RE1: Regex = Regex::new(r"(?P<a>(?:[^'</]+/)+)+").unwrap();
    }

    let prev_path = RE1.replace(text, "$a").to_string();

    let mut items: Vec<CompletionItem> = vec![];

    for inc_uri in request.store.documents.keys() {
        for inc_uri_folder in inc_uri_folders.iter() {
            if inc_uri
                .to_string()
                .contains(&format!("{}/{}", inc_uri_folder, prev_path))
            {
                if let Ok(inc_path) = inc_uri.to_file_path() {
                    let parent_folder = inc_uri_folder.to_file_path().unwrap().join(text);
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
                        if !use_chevron {
                            trail = "\"";
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
            let tmp_path = inc_folder_path.join(text);
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

    Some(CompletionList {
        items,
        ..Default::default()
    })
}
