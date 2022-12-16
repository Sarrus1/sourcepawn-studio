use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use lsp_types::{CompletionItem, CompletionList, CompletionParams, Position, Url};
use regex::Regex;

use crate::{
    spitem::{get_all_items, get_items_from_position, SPItem},
    utils::range_contains_pos,
};

use super::FeatureRequest;

pub fn provide_completions(request: FeatureRequest<CompletionParams>) -> Option<CompletionList> {
    let document = request.store.get(&request.uri)?;
    let all_items = get_all_items(&request.store)?;
    let position = request.params.text_document_position.position;
    let line = document.line(position.line)?;

    if !is_method_call(line, position) {
        let mut items: Vec<CompletionItem> = Vec::new();
        for sp_item in all_items.iter() {
            let res = sp_item.lock().unwrap().to_completion(&request.params);
            if let Some(res) = res {
                items.push(res);
            }
        }

        return Some(CompletionList {
            items,
            ..Default::default()
        });
    }

    let words = get_line_words(line, position);
    for word in words.into_iter().flatten().rev() {
        let word_pos = Position {
            line: word.range.start.line,
            character: ((word.range.start.character + word.range.end.character) / 2),
        };
        let items = get_items_from_position(
            &request.store,
            word_pos,
            request
                .params
                .text_document_position
                .text_document
                .uri
                .clone(),
        );
        if items.is_empty() {
            continue;
        }
        for item in items.iter() {
            let type_ = item.lock().unwrap().type_();
            for item_ in all_items.iter() {
                if item_.lock().unwrap().name() == type_ {
                    let item_lock = item_.lock().unwrap().clone();
                    match item_lock {
                        SPItem::Methodmap(mm_item) => {
                            return Some(CompletionList {
                                // TODO: Handle inherit here
                                items: get_children_of_mm_or_es(
                                    &all_items,
                                    mm_item.name,
                                    request.params,
                                ),
                                ..Default::default()
                            });
                        }
                        SPItem::EnumStruct(es_item) => {
                            return Some(CompletionList {
                                items: get_children_of_mm_or_es(
                                    &all_items,
                                    es_item.name,
                                    request.params,
                                ),
                                ..Default::default()
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    None
}

fn find_func(
    all_items: &Vec<Arc<Mutex<SPItem>>>,
    uri: Arc<Url>,
    pos: Position,
) -> Option<Arc<Mutex<SPItem>>> {
    for item in all_items {
        if let SPItem::Function(function_item) = &*item.lock().unwrap() {
            if function_item.uri.eq(&uri) && range_contains_pos(function_item.full_range, pos) {
                return Some(item.clone());
            }
        }
    }

    None
}

fn is_method_call(line: &str, pos: Position) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?:\.|::)\w*$").unwrap();
    }
    let sub_line: String = line.chars().take(pos.character as usize).collect();
    RE.is_match(&sub_line)
}

#[derive(Debug)]
pub struct MatchToken {
    pub text: String,
    pub range: lsp_types::Range,
}

fn get_line_words(line: &str, pos: Position) -> Vec<Option<MatchToken>> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\w+").unwrap();
    }
    let sub_line: String = line.chars().take(pos.character as usize).collect();

    let mut res: Vec<Option<MatchToken>> = vec![];
    for caps in RE.captures_iter(&sub_line) {
        res.push(caps.get(0).map(|m| MatchToken {
            text: m.as_str().to_string(),
            range: lsp_types::Range {
                start: Position {
                    line: pos.line,
                    character: m.start() as u32,
                },
                end: Position {
                    line: pos.line,
                    character: m.end() as u32,
                },
            },
        }));
    }

    res
}

fn get_children_of_mm_or_es(
    all_item: &[Arc<Mutex<SPItem>>],
    parent_name: String,
    params: CompletionParams,
) -> Vec<CompletionItem> {
    let mut res: Vec<CompletionItem> = vec![];
    for item in all_item.iter() {
        let item_lock = item.lock().unwrap();
        if let Some(parent_) = item_lock.parent() {
            if parent_name != parent_.lock().unwrap().name() {
                continue;
            }
            if let Some(completion) = item_lock.to_completion(&params) {
                res.push(completion);
            }
        }
    }

    res
}
