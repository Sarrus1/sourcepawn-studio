use std::sync::{Arc, Mutex};

use lsp_types::{CompletionItem, CompletionList, CompletionParams, Position};

use crate::{
    providers::FeatureRequest,
    spitem::{get_items_from_position, SPItem},
};

use super::context::get_line_words;

/// Search in a vector of items for the childs of a type and return the associated
/// vector of [CompletionItem](lsp_types::CompletionItem).
///
/// # Arguments
///
/// * `all_items` - Vector of [SPItem](crate::spitem::SPItem).
/// * `parent_name` - Name of the parent.
/// * `params` - [Parameters](lsp_types::completion::CompletionParams) of the completion request.
pub(super) fn get_children_of_mm_or_es(
    all_items: &[Arc<Mutex<SPItem>>],
    parent_name: String,
    params: CompletionParams,
) -> Vec<CompletionItem> {
    let mut res: Vec<CompletionItem> = vec![];
    for item in all_items.iter() {
        let item_lock = item.lock().unwrap();
        if let Some(parent_) = item_lock.parent() {
            if parent_name != parent_.lock().unwrap().name() {
                continue;
            }
            if let Some(completion) = item_lock.to_completion(&params, true) {
                res.push(completion);
            }
        }
    }

    res
}

/// Return a [CompletionList](lsp_types::CompletionList) of all non method completions (that don't come
/// after a `.` or `::`).
///
/// # Arguments
///
/// * `all_items` - Vector of [SPItem](crate::spitem::SPItem).
/// * `params` - [Parameters](lsp_types::completion::CompletionParams) of the completion request.
pub(super) fn get_non_method_completions(
    all_items: Vec<Arc<Mutex<SPItem>>>,
    params: CompletionParams,
) -> Option<CompletionList> {
    let mut items: Vec<CompletionItem> = Vec::new();
    for sp_item in all_items.iter() {
        let res = sp_item.lock().unwrap().to_completion(&params, false);
        if let Some(res) = res {
            items.push(res);
        }
    }

    Some(CompletionList {
        items,
        ..Default::default()
    })
}

/// Return a [CompletionList](lsp_types::CompletionList) of all method completions (that should come
/// after a `.` or `::`).
///
/// # Arguments
///
/// * `all_items` - Vector of [SPItem](crate::spitem::SPItem).
/// * `sub_line` - Sub line of the document to analyze.
/// * `position` - [Position](lsp_types::Position) of the request.
/// * `params` - [Parameters](lsp_types::completion::CompletionParams) of the completion request.
pub(super) fn get_method_completions(
    all_items: Vec<Arc<Mutex<SPItem>>>,
    sub_line: &str,
    position: Position,
    request: FeatureRequest<CompletionParams>,
) -> Option<CompletionList> {
    let words = get_line_words(sub_line, position);
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
                if item_.lock().unwrap().name() != type_ {
                    continue;
                }
                let item_lock = item_.lock().unwrap().clone();
                match item_lock {
                    SPItem::Methodmap(mm_item) => {
                        return Some(CompletionList {
                            // TODO: Handle inherit here
                            // TODO: Handle static methods
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

    None
}
