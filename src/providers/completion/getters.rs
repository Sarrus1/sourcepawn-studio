use std::sync::{Arc, RwLock};

use lsp_types::{CompletionItem, CompletionList, CompletionParams, Position};

use crate::{
    providers::FeatureRequest,
    spitem::{get_items_from_position, SPItem},
};

use super::context::get_line_words;

/// Return a [CompletionList](lsp_types::CompletionList) of all non method completions (that don't come
/// after a `.` or `::`).
///
/// # Arguments
///
/// * `all_items` - Vector of [SPItem](crate::spitem::SPItem).
/// * `params` - [Parameters](lsp_types::completion::CompletionParams) of the completion request.
pub(super) fn get_non_method_completions(
    all_items: Vec<Arc<RwLock<SPItem>>>,
    params: CompletionParams,
) -> Option<CompletionList> {
    let mut items: Vec<CompletionItem> = Vec::new();
    for sp_item in all_items.iter() {
        let res = sp_item.read().unwrap().to_completions(&params, false);
        items.extend(res);
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
    all_items: Vec<Arc<RwLock<SPItem>>>,
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
            let type_ = item.read().unwrap().type_();
            for type_item in all_items.iter() {
                if type_item.read().unwrap().name() != type_ {
                    continue;
                }
                let item_lock = type_item.read().unwrap().clone();
                match item_lock {
                    SPItem::Methodmap(mm_item) => {
                        let mut items = vec![];
                        for child in mm_item.children.iter() {
                            match &*child.read().unwrap() {
                                SPItem::Function(method_item) => {
                                    if item.read().unwrap().name()
                                        == type_item.read().unwrap().name()
                                    {
                                        if method_item.is_static() {
                                            // We are trying to call static methods.
                                            items.extend(
                                                method_item.to_completions(&request.params, true),
                                            );
                                        }
                                        continue;
                                    } else if !method_item.is_static() {
                                        // We are trying to call non static methods.
                                        items.extend(
                                            method_item.to_completions(&request.params, true),
                                        );
                                    }
                                }
                                SPItem::Property(property_item) => {
                                    items.extend(property_item.to_completion(&request.params, true))
                                }
                                _ => {}
                            }
                        }
                        return Some(CompletionList {
                            // TODO: Handle inherit here
                            items,
                            ..Default::default()
                        });
                    }
                    SPItem::EnumStruct(es_item) => {
                        let mut items = vec![];
                        for child in es_item.children.iter() {
                            items.extend(
                                child.read().unwrap().to_completions(&request.params, true),
                            );
                        }
                        return Some(CompletionList {
                            items,
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
